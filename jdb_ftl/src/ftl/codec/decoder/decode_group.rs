use super::bits::read_bits;
use crate::ftl::{
  codec::zigzag_decode,
  frame::Head,
  seg::{GroupHeader, Seg},
};

/// Decode an entire group of compressed PPAs into a provided buffer.
pub fn decode_group(header: Head, payload: &[u8], out: &mut [u64]) {
  if payload.is_empty() {
    out.fill(u64::MAX);
    return;
  }

  let group_size = out.len();
  // Read valid item count (n_valid) from the first 2 bytes.
  let n_valid = unsafe { u16::from_le_bytes(*(payload.as_ptr() as *const [u8; 2])) } as usize;
  if n_valid == 0 {
    out.fill(u64::MAX);
    return;
  }

  // Initialize EfViewU16 for sparse index.
  let ef_index_payload = &payload[2..];
  let ef_view = crate::ftl::codec::ef::EfViewU16::new(ef_index_payload, n_valid, group_size);
  let ef_bytes = 2 + crate::ftl::codec::ef::byte_len(n_valid, group_size);
  let payload = &payload[ef_bytes..]; // Shadow with PGM payload

  // Support for legacy/special Direct Mode.
  if header.is_direct() {
    let count = header.count() as usize;
    let width = header.width();
    let base_len = header.base_len() as usize;

    unsafe {
      let mut base = 0u64;
      let base_ptr = payload.as_ptr();
      std::ptr::copy_nonoverlapping(base_ptr, &mut base as *mut u64 as *mut u8, base_len);

      let residuals = payload.get_unchecked(base_len..);
      for i in 0..count {
        let delta = read_bits(residuals, i * width as usize, width);
        *out.get_unchecked_mut(i) = base.wrapping_add(delta);
      }
    }
  } else {
    // Standard PGM (formerly V2)
    // Residual-Patch Decoder Path
    let g_header = unsafe { GroupHeader::from_bytes(payload) };
    let mode = g_header.mode();

    if mode == 0 {
      // Mode 0: Raw
      unsafe {
        let src_ptr = payload.as_ptr().add(GroupHeader::SIZE) as *const u64;
        std::ptr::copy_nonoverlapping(src_ptr, out.as_mut_ptr(), n_valid);
      }
    } else {
      let num_segs = g_header.seg_count() as usize;
      let num_outliers = g_header.outlier_count() as usize;
      let outlier_bw = g_header.outlier_bw();

      let segments = Seg::view_table(payload, num_outliers, num_segs);
      let seg_start_idxs = Seg::view_seg_ef(payload, num_outliers, num_segs);

      // 1. Initial reconstruction
      for seg_idx in 0..num_segs {
        let seg = unsafe { segments.get_unchecked(seg_idx) };
        let start_idx = seg_start_idxs.get(seg_idx) as usize;
        let slope = seg.slope() as i64;
        let base = seg.base();
        let width = seg.bit_width();

        let end_idx = if seg_idx == num_segs - 1 {
          n_valid
        } else {
          seg_start_idxs.get(seg_idx + 1) as usize
        };

        unsafe {
          let seg_out = out.get_unchecked_mut(start_idx..end_idx);
          let mut acc = 0i64;

          if width == 0 {
            for val in seg_out.iter_mut() {
              *val = base.wrapping_add((acc >> 24) as u64);
              acc += slope;
            }
          } else {
            let byte_offset = seg.bit_offset() as usize;
            let seg_residuals = payload.get_unchecked(byte_offset..);
            let w = width as usize;
            for (i, val) in seg_out.iter_mut().enumerate() {
              let pred = base.wrapping_add((acc >> 24) as u64);
              acc += slope;
              let bit_idx = i.wrapping_mul(w);
              let delta = read_bits(seg_residuals, bit_idx, width);
              *val = pred.wrapping_add(delta);
            }
          }
        }
      }

      // 2. Outliers
      if num_outliers > 0 {
        let outlier_idxs = Seg::view_outlier_ef(payload, num_outliers);
        let mut pgm_bit_end = 0;
        for (seg_idx, seg_p) in segments.iter().enumerate().take(num_segs) {
          if seg_p.bit_width() > 0 {
            let start = seg_p.bit_offset() as usize * 8;
            let len = if seg_idx == num_segs - 1 {
              n_valid - seg_start_idxs.get(seg_idx) as usize
            } else {
              seg_start_idxs.get(seg_idx + 1) as usize - seg_start_idxs.get(seg_idx) as usize
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
          let aligned_offset = (offset + 1) & !1;
          (aligned_offset + num_segs * Seg::METADATA_SIZE) * 8
        } else {
          pgm_bit_end
        };

        for i in 0..num_outliers {
          let lba_off = outlier_idxs.get(i) as usize;
          let bit_idx = outlier_bit_start + i * outlier_bw as usize;
          let zigzag_res = read_bits(payload, bit_idx, outlier_bw);
          let res = zigzag_decode(zigzag_res);

          let mut seg_idx = 0;
          for s in 0..num_segs {
            if seg_start_idxs.get(s) as usize <= lba_off {
              seg_idx = s;
            } else {
              break;
            }
          }

          let seg = &segments[seg_idx];
          let rel_idx = lba_off - seg_start_idxs.get(seg_idx) as usize;
          let pred = seg
            .base()
            .wrapping_add(((rel_idx as i64).wrapping_mul(seg.slope() as i64) >> 24) as u64);
          out[lba_off] = pred.wrapping_add(res as u64);
        }
      }
    }
  }

  // Optimized in-place scattering:
  // Since EF index is non-decreasing, and logical_idx >= dense_idx (i),
  // we can scatter backward to avoid overwriting and extra allocations.
  let mut next_logical_idx = group_size;
  for i in (0..n_valid).rev() {
    let logical_idx = ef_view.get(i) as usize;
    let val = out[i];

    // Fill gaps from logical_idx+1 to next_logical_idx with MAX.
    if next_logical_idx > logical_idx + 1 {
      out[logical_idx + 1..next_logical_idx].fill(u64::MAX);
    }

    out[logical_idx] = val;
    next_logical_idx = logical_idx;
  }
  // Fill leading gaps.
  if next_logical_idx > 0 {
    out[0..next_logical_idx].fill(u64::MAX);
  }
}
