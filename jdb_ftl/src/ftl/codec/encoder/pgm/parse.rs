use super::types::OldSegInfo;
use crate::ftl::seg::{GroupHeader, Seg};

pub fn parse_old_structure(
  old_pgm_payload: Option<&[u8]>,
  n: usize,
) -> (Vec<OldSegInfo<'_>>, Vec<u16>) {
  let mut old_segs_info = Vec::new();
  let mut old_outliers_idxs = Vec::new();

  if let Some(payload) = old_pgm_payload
    && payload.len() >= GroupHeader::SIZE
  {
    let gh = unsafe { GroupHeader::from_bytes(payload) };
    if gh.mode() != 0 {
      let num_segs = gh.seg_count() as usize;
      let num_outliers = gh.outlier_count() as usize;

      let segments = Seg::view_table(payload, num_outliers, num_segs);
      let seg_start_idxs = Seg::view_seg_ef(payload, num_outliers, num_segs);
      let outliers_ef = Seg::view_outlier_ef(payload, num_outliers);

      for i in 0..num_outliers {
        old_outliers_idxs.push(outliers_ef.get(i));
      }

      for i in 0..num_segs {
        let seg = unsafe { *segments.get_unchecked(i) };
        let start = seg_start_idxs.get(i);
        let end = if i + 1 < num_segs {
          seg_start_idxs.get(i + 1)
        } else {
          n as u16
        };
        if start < end {
          // Calculate byte range for reuse
          let bit_width = seg.bit_width() as usize;
          let len_points = (end - start) as usize;
          let bits = len_points * bit_width;
          let len_bytes = bits.div_ceil(8);
          let byte_offset = seg.bit_offset() as usize;

          old_segs_info.push(OldSegInfo {
            seg,
            start,
            end,
            byte_offset,
            len_bytes,
            _phantom: std::marker::PhantomData,
          });
        }
      }
    }
  }
  (old_segs_info, old_outliers_idxs)
}
