#[derive(Debug, Clone, Copy)]
pub struct FitResult {
  /// Base value for predictions. 预测的基础值。
  pub base: u64,
  /// Fixed-point slope (scaled by 2^24). 定点斜率（按 2^24 缩放）。
  pub slope: i32,
  /// Length of the fitted segment. 拟合段的长度。
  pub length: usize,
  /// Maximum residual in this segment. 该段中的最大残差。
  pub max_residual: u64,
}

/// Pure integer implementation of the PGM building algorithm.
/// 纯整数实现的 PGM 构建算法。
/// Ensures no i128 overflow even with large PPA spans.
/// 保证在 PPA 跨度很大时也不会发生 i128 溢出。
pub fn find_longest_segment(values: &[u64], epsilon: u64) -> (FitResult, Vec<usize>) {
  if values.is_empty() {
    return (
      FitResult {
        base: 0,
        slope: 0,
        length: 0,
        max_residual: 0,
      },
      Vec::new(),
    );
  }
  if values.len() == 1 {
    return (
      FitResult {
        base: values[0],
        slope: 0,
        length: 1,
        max_residual: 0,
      },
      Vec::new(),
    );
  }

  let first_val = values[0] as i128;
  let eps = epsilon as i128;

  let mut min_num = i128::MIN;
  let mut min_den = 1i128;

  let mut max_num = i128::MAX;
  let mut max_den = 1i128;

  let mut best_len = 1;
  let is_max_segment = values[0] == u64::MAX;

  let mut outliers = Vec::new();
  let mut iter = values.iter().enumerate().skip(1);

  while let Some((i, &val)) = iter.next() {
    // Enforce segmentation on u64::MAX (deletion) boundary.
    let current_is_max = val == u64::MAX;
    if current_is_max != is_max_segment {
      break;
    }

    let x = i as i128;
    let y = val as i128;

    let cur_low_num = y - first_val - eps;
    let cur_high_num = y - first_val + eps;

    // Check Compatibility

    // New Min Slope > Current Max Slope?
    let new_min_violates = if max_num != i128::MAX {
      cur_low_num * max_den > max_num * x
    } else {
      false
    };

    // New Max Slope < Current Min Slope?
    let new_max_violates = if min_num != i128::MIN {
      cur_high_num * min_den < min_num * x
    } else {
      false
    };

    if !new_min_violates && !new_max_violates {
      // Consistent. Update cone.
      if min_num == i128::MIN || cur_low_num * min_den > min_num * x {
        min_num = cur_low_num;
        min_den = x;
      }
      if max_num == i128::MAX || cur_high_num * max_den < max_num * x {
        max_num = cur_high_num;
        max_den = x;
      }
      best_len = i + 1;
    } else {
      // Inconsistent. Try to skip?
      // Lookahead Strategy:
      // Check if the NEXT TWO points (consensus) fit the CURRENT cone.
      // If so, `i` is an isolated outlier. Skip it.
      let mut lookahead = iter.clone();

      // Peek i+1
      if let Some((next_i, &next_val)) = lookahead.next() {
        // Boundary check for i+1
        let same_type = (next_val == u64::MAX) == is_max_segment;

        if same_type {
          // Check if i+1 fits cone
          let nx = next_i as i128;
          let ny = next_val as i128;
          let n_low = ny - first_val - eps;
          let n_high = ny - first_val + eps;

          let n_min_violates = if max_num != i128::MAX {
            n_low * max_den > max_num * nx
          } else {
            false
          };
          let n_max_violates = if min_num != i128::MIN {
            n_high * min_den < min_num * nx
          } else {
            false
          };

          if !n_min_violates && !n_max_violates {
            // i+1 FITS. Now robustness check: check i+2
            let mut plus2_confirm = true; // Assume true if end of stream (i+1 is last point)

            if let Some((next2_i, &next2_val)) = lookahead.next() {
              let type2 = (next2_val == u64::MAX) == is_max_segment;
              if !type2 {
                plus2_confirm = false; // Boundary change at i+2, conservative break
              } else {
                // Check if i+2 fits cone
                let nx2 = next2_i as i128;
                let ny2 = next2_val as i128;
                let n2_low = ny2 - first_val - eps;
                let n2_high = ny2 - first_val + eps;

                let n2_min_violates = if max_num != i128::MAX {
                  n2_low * max_den > max_num * nx2
                } else {
                  false
                };
                let n2_max_violates = if min_num != i128::MIN {
                  n2_high * min_den < min_num * nx2
                } else {
                  false
                };

                if n2_min_violates || n2_max_violates {
                  // i+2 VIOLATES. This means i+1 might be the start of a new trend, not just `i` being an outlier.
                  // Don't skip `i`.
                  plus2_confirm = false;
                }
              }
            }

            if plus2_confirm {
              outliers.push(i);
              continue;
            }
          }
        }
      }
      // If we reach here, we couldn't skip.
      break;
    }
  }

  // Calculate final slope: avg((min + max) / 2) * 2^24.
  let final_slope_scaled: i32 = if best_len > 1 {
    let s_min = (min_num << 24) / min_den;
    let s_max = (max_num << 24) / max_den;
    let avg = (s_min + s_max) / 2;
    avg.clamp(i32::MIN as i128, i32::MAX as i128) as i32
  } else {
    0
  };

  // Find optimal base
  let mut min_diff = i64::MAX;
  let mut max_diff = i64::MIN;
  let mut acc = 0i64;
  let s64 = final_slope_scaled as i64;

  // We must iterate 0..best_len to update `acc`, but only check diffs for non-outliers.
  let mut outlier_iter = outliers.iter().peekable();

  for (i, &val) in values.iter().enumerate().take(best_len) {
    let is_outlier = outlier_iter.peek().is_some_and(|&&idx| idx == i);
    if is_outlier {
      outlier_iter.next();
      // Skip diff check, but update acc
      acc += s64;
      continue;
    }

    let pred = (first_val as i64).wrapping_add(acc >> 24);
    let diff = (val as i64).wrapping_sub(pred);
    if diff < min_diff {
      min_diff = diff;
    }
    if diff > max_diff {
      max_diff = diff;
    }
    acc += s64;
  }

  let max_residual = if max_diff >= min_diff {
    (max_diff - min_diff) as u64
  } else {
    0
  };

  (
    FitResult {
      base: (values[0] as i64).wrapping_add(min_diff) as u64,
      slope: final_slope_scaled,
      length: best_len,
      max_residual,
    },
    outliers,
  )
}
