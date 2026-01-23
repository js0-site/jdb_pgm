use super::types::{OldSegInfo, Plan};
use crate::ftl::codec::optimal::find_longest_segment;

pub fn generate_plan(
  group_ppas: &[u64],
  dirty_map: &[bool],
  epsilon: usize,
  old_segs_info: &[OldSegInfo],
  old_outliers_idxs: &[u16],
  skip_threshold: usize,
) -> (Vec<Plan>, Vec<(u16, u64)>) {
  let n = group_ppas.len();
  let mut plan = Vec::new();
  let mut outliers = Vec::new();
  let mut cursor = 0;
  let mut old_cursor = 0;

  while cursor < n {
    // Try Reuse
    let mut reused = false;
    if old_cursor < old_segs_info.len() {
      let old = &old_segs_info[old_cursor];
      // Only reuse if exact match on start
      if old.start as usize == cursor {
        // Check dirty
        let len = (old.end - old.start) as usize;
        let range_dirty = dirty_map[cursor..cursor + len].iter().any(|&x| x);

        if !range_dirty {
          // Check next boundary
          let next_start = cursor + len;
          let next_boundary_dirty = if next_start < n {
            dirty_map[next_start] // Check single point at start of next
          } else {
            false
          };

          if !next_boundary_dirty {
            // Full Reuse
            plan.push(Plan::Reuse {
              old_idx: old_cursor,
            });
            // Outliers will be migrated in a separate pass using Two-Pointers.
            cursor += len;
            old_cursor += 1;
            reused = true;
          } else {
            // Smart Merge Probe skipped
          }
        } else {
          // Range dirty -> Refit. old_cursor consumes this segment but we don't reuse it.
          old_cursor += 1;
        }
      } else if (old.start as usize) < cursor {
        // Should not happen if we sync cursors, but fast forward old
        old_cursor += 1;
        continue;
      }
    }

    if !reused {
      // Greedy Refit
      let (fit, skipped) = find_longest_segment(&group_ppas[cursor..], epsilon as u64);

      if fit.length.saturating_sub(skipped.len()) < skip_threshold && cursor + fit.length < n {
        // Too short, turn into outliers
        outliers.push((cursor as u16, group_ppas[cursor]));
        cursor += 1;
      } else {
        for &offset in &skipped {
          let abs_idx = cursor + offset;
          outliers.push((abs_idx as u16, group_ppas[abs_idx]));
        }

        // Calculate max residual for bit width
        let mut max_res = 0;
        if fit.max_residual > 0 {
          max_res = fit.max_residual;
        }

        plan.push(Plan::New {
          fit,
          len: fit.length as u16,
          max_res,
        });
        cursor += fit.length;
      }
    }
  }

  // 2.5 Migrate Outliers for Reused Segments (Two Pointers)
  {
    let mut o_ptr = 0; // Pointer to old_outliers_idxs
    let mut current_start = 0;

    for p in &plan {
      match p {
        Plan::Reuse { old_idx } => {
          let old = &old_segs_info[*old_idx];
          let old_seg_start = old.start;
          let old_seg_end = old.end;

          // Collect outliers belonging to this reused segment
          while o_ptr < old_outliers_idxs.len() {
            let o_idx = old_outliers_idxs[o_ptr];
            if o_idx < old_seg_start {
              o_ptr += 1; // Skip outliers before this segment
            } else if o_idx < old_seg_end {
              // Match!
              outliers.push((o_idx, group_ppas[o_idx as usize]));
              o_ptr += 1;
            } else {
              break;
            }
          }
          current_start += old.end - old.start;
        }
        Plan::New { len, .. } => {
          // New segments have their outliers already collected during greedy refit.
          // Just skip potential old outliers that might have been in this range (implied drop)
          let new_end = current_start + *len;
          while o_ptr < old_outliers_idxs.len() && old_outliers_idxs[o_ptr] < new_end {
            o_ptr += 1;
          }
          current_start += *len;
        }
      }
    }
  }

  // Sort outliers by index
  outliers.sort_unstable_by_key(|k| k.0);

  (plan, outliers)
}
