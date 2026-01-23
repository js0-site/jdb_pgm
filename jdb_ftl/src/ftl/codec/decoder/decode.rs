use crate::ftl::{
  codec::{read_bits, zigzag_decode},
  frame::Head,
  seg::{GroupHeader, Seg},
};

pub fn decode(header: Head, sub_idx: usize, payload: &[u8], group_size: usize) -> u64 {
  if payload.is_empty() {
    return u64::MAX;
  }

  // 1. Read valid item count (n_valid)
  let n_valid = unsafe { u16::from_le_bytes(*(payload.as_ptr() as *const [u8; 2])) } as usize;
  if n_valid == 0 {
    return u64::MAX;
  }

  // 2. Map logical index (sub_idx) to dense index using EF index
  let ef_index_payload = &payload[2..];
  let ef_view = crate::ftl::codec::ef::EfViewU16::new(ef_index_payload, n_valid, group_size);

  let (dense_idx, logical_val) = ef_view.predecessor(sub_idx as u16);
  if logical_val != sub_idx as u16 {
    return u64::MAX;
  }

  let ef_bytes = 2 + crate::ftl::codec::ef::byte_len(n_valid, group_size);
  let payload = &payload[ef_bytes..];

  // 3. Direct Mode Decoding
  if header.is_direct() {
    let width = header.width() as usize;
    let base_len = header.base_len() as usize;
    unsafe {
      let mut base = 0u64;
      let base_ptr = payload.as_ptr();
      std::ptr::copy_nonoverlapping(base_ptr, &mut base as *mut u64 as *mut u8, base_len);
      let residuals = payload.get_unchecked(base_len..);
      let delta = read_bits(residuals, dense_idx * width, width as u8);
      return base.wrapping_add(delta);
    }
  }

  // 4. Standard PGM Decoder Path (Residual-Patch)
  let g_header = unsafe { GroupHeader::from_bytes(payload) };
  if g_header.mode() == 0 {
    // Raw fallback (Dense)
    unsafe {
      let src_ptr = payload.as_ptr().add(GroupHeader::SIZE + dense_idx * 8) as *const u64;
      return *src_ptr;
    }
  }

  let num_segs = g_header.seg_count() as usize;
  let num_outliers = g_header.outlier_count() as usize;
  let outlier_bw = g_header.outlier_bw();
  let seg_start_idxs = Seg::view_seg_ef(payload, num_outliers, num_segs);

  // Check outliers first (Fast Path)
  if num_outliers > 0 {
    let outlier_idxs = Seg::view_outlier_ef(payload, num_outliers);
    let (idx, val) = outlier_idxs.predecessor(dense_idx as u16);
    if val == dense_idx as u16 {
      // Find pred for this outlier (using dense_idx)
      let (seg_idx, start_idx_val) = seg_start_idxs.predecessor(dense_idx as u16);
      let segments = Seg::view_table(payload, num_outliers, num_segs);
      let seg = &segments[seg_idx];
      let i = (dense_idx as u32).wrapping_sub(start_idx_val as u32);
      let pred = seg
        .base()
        .wrapping_add(((i as i64).wrapping_mul(seg.slope() as i64) >> 24) as u64);

      // Locate outlier residual
      let mut pgm_bit_end = 0;
      for (s, seg_p) in segments.iter().enumerate().take(num_segs) {
        if seg_p.bit_width() > 0 {
          let start = seg_p.bit_offset() as usize * 8;
          let len = if s == num_segs - 1 {
            n_valid - seg_start_idxs.get(s) as usize
          } else {
            seg_start_idxs.get(s + 1) as usize - seg_start_idxs.get(s) as usize
          };
          let end = start + len * seg_p.bit_width() as usize;
          if end > pgm_bit_end {
            pgm_bit_end = end;
          }
        }
      }
      let outlier_bit_start = if pgm_bit_end == 0 {
        let offset =
          GroupHeader::SIZE + Seg::ef_len_bytes(num_outliers) + Seg::ef_len_bytes(num_segs);
        (((offset + 1) & !1) + num_segs * Seg::METADATA_SIZE) * 8
      } else {
        pgm_bit_end
      };

      let bit_idx = outlier_bit_start + idx * outlier_bw as usize;
      let zigzag_res = read_bits(payload, bit_idx, outlier_bw);
      return pred.wrapping_add(zigzag_decode(zigzag_res) as u64);
    }
  }

  // PGM Path for V2
  let (seg_idx, start_idx_val) = seg_start_idxs.predecessor(dense_idx as u16);
  let segments = Seg::view_table(payload, num_outliers, num_segs);
  let seg = &segments[seg_idx];
  let i = (dense_idx as u32).wrapping_sub(start_idx_val as u32);
  let pred = seg
    .base()
    .wrapping_add(((i as i64).wrapping_mul(seg.slope() as i64) >> 24) as u64);
  let width = seg.bit_width();
  if width == 0 {
    pred
  } else {
    let byte_offset = seg.bit_offset() as usize;
    let bit_idx = (i as usize).wrapping_mul(width as usize);
    unsafe {
      let delta = read_bits(payload.get_unchecked(byte_offset..), bit_idx, width);
      pred.wrapping_add(delta)
    }
  }
}
