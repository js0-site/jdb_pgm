use crate::ftl::{conf::Conf, frame::Head, l1::L1, seg::Seg};

#[derive(Debug, Default)]
pub struct FtlStats {
  pub segment_lengths: Vec<usize>,
  pub total_bytes_physical: usize,
  pub total_bytes_logical: usize,
  pub group_count_empty: usize,
  pub group_count_direct: usize,
  pub group_count_pgm: usize,
  pub group_count_raw: usize,
  // Simulation / Optimization Stats
  // 模拟/优化统计
  pub segment_bit_widths: Vec<u8>,
  pub immediate_mode_candidates: usize,
  pub payload_bytes_saved: usize,

  // Polymorphic Compression Simulation
  pub type_a_count: usize, // Slope=1, Width=0
  pub type_b_count: usize, // Slope=0
  pub linear_model_bytes_saved: usize,

  // Exception Table Simulation
  pub exception_segment_count: usize,
  pub exception_table_bytes_saved: usize,

  // Residual Sparsity
  pub total_residuals: usize,
  pub zero_residuals: usize,

  // PFOR Simulation
  pub pfor_bytes_saved: usize,
  pub pfor_candidate_segments: usize,

  // Actual Outlier Storage Overhead
  pub outlier_bytes: usize,
}

impl FtlStats {
  pub fn collect<C: Conf>(l1: &L1) -> Self {
    let mut stats = Self::default();

    let mut buf = vec![0u64; C::GROUP_SIZE];

    for group in l1.groups.iter() {
      let storage = &group.storage;
      let phys_len = storage.len();
      stats.total_bytes_physical += phys_len;

      if storage.is_empty() {
        stats.group_count_empty += 1;
        continue;
      }

      let header = unsafe { Head::from_bytes(storage) };
      let payload = unsafe { storage.get_unchecked(Head::SIZE..) };

      // Decode to get valid items count
      crate::ftl::codec::decode_group(*header, payload, &mut buf);
      let valid_items = buf.iter().filter(|&&v| v != u64::MAX).count();
      stats.total_bytes_logical += valid_items * 8; // 8 bytes per PBA

      if header.is_empty() {
        stats.group_count_empty += 1;
        continue;
      }

      if header.is_direct() {
        stats.group_count_direct += 1;
        continue;
      }

      // Skip Elias-Fano sparse index for manual PGM inspection
      let n_valid = unsafe { u16::from_le_bytes(*(payload.as_ptr() as *const [u8; 2])) } as usize;
      let ef_bytes = 2 + crate::ftl::codec::ef::byte_len(n_valid, C::GROUP_SIZE);
      let payload = unsafe { payload.get_unchecked(ef_bytes..) };

      let (outlier_num, seg_num, mode) = {
        let gh = unsafe { crate::ftl::seg::GroupHeader::from_bytes(payload) };
        (
          gh.outlier_count() as usize,
          gh.seg_count() as usize,
          gh.mode(),
        )
      };

      if mode == 0 {
        stats.group_count_raw += 1;
        continue;
      }

      stats.group_count_pgm += 1;

      let start_idxs_raw = Seg::view_seg_ef(payload, outlier_num, seg_num);
      let mut start_idxs: Vec<_> = start_idxs_raw.iter().collect();
      start_idxs.sort_unstable();

      let actual_seg_num = start_idxs.len();
      if actual_seg_num == 0 {
        continue;
      }

      // Calculate lengths
      for i in 0..actual_seg_num - 1 {
        stats
          .segment_lengths
          .push((start_idxs[i + 1] - start_idxs[i]) as usize);
      }

      // Last segment length
      let last_start = start_idxs[actual_seg_num - 1] as usize;
      if C::GROUP_SIZE >= last_start {
        stats.segment_lengths.push(C::GROUP_SIZE - last_start);
      }

      let segs_table = Seg::view_table(payload, outlier_num, seg_num);
      for i in 0..actual_seg_num.min(segs_table.len()) {
        // Calculate length for this segment on-the-fly for simulation
        let len = if i < actual_seg_num - 1 {
          (start_idxs[i + 1] - start_idxs[i]) as usize
        } else {
          C::GROUP_SIZE.saturating_sub(start_idxs[i] as usize)
        };

        if i < segs_table.len() {
          let seg = segs_table[i];
          let bit_width = seg.bit_width();
          let slope = seg.slope();

          stats.segment_bit_widths.push(bit_width);

          // Simulation: Immediate Mode Candidates
          if slope == 0 {
            let total_bits = (len as u64) * (bit_width as u64);
            if total_bits <= 42 {
              stats.immediate_mode_candidates += 1;
              let original_payload_bytes = (total_bits as usize).div_ceil(8);
              stats.payload_bytes_saved += original_payload_bytes;
            }
          }

          // Exception Table Simulation:
          // If moving 1 outlier to a 8-byte KV entry saves space.
          if bit_width > 4 && len > 4 {
            let bit_start = seg.bit_offset() as usize;
            let mut residuals = Vec::with_capacity(len);
            for j in 0..len {
              let r = crate::ftl::codec::decoder::read_bits(
                payload,
                bit_start + j * bit_width as usize,
                bit_width,
              );
              stats.total_residuals += 1;
              if r == 0 {
                stats.zero_residuals += 1;
              }
              residuals.push(r);
            }

            // PFOR Simulation: Try all bit widths B < current bit_width
            let original_bits = (len as u64) * (bit_width as u64);
            let mut min_bits = original_bits;

            // Simple Exception Table (1 outlier) is a subset of PFOR.
            // We test every possible bit width to find the optimal 'B'.
            for b in 0..bit_width {
              let threshold = 1u64.checked_shl(b as u32).unwrap_or(u64::MAX);
              let outliers = residuals.iter().filter(|&&r| r >= threshold).count();
              // Cost = main area (len * b) + exception area (outliers * 72 bits)
              // 72 bits = 64 bits (PBA) + 8 bits (Index within segment)
              let cost = (len as u64) * (b as u64) + (outliers as u64) * 72;
              if cost < min_bits {
                min_bits = cost;
              }
            }

            if min_bits < original_bits {
              stats.pfor_candidate_segments += 1;
              stats.pfor_bytes_saved += ((original_bits - min_bits) / 8) as usize;
            }
          }

          // Polymorphic Compression Simulation (Type A/B)
          if bit_width == 0 {
            if slope == 1 {
              // Type A: Linear run.
              stats.type_a_count += 1;
              stats.linear_model_bytes_saved += 6;
            } else if slope == 0 {
              // Type B: Constant run.
              stats.type_b_count += 1;
              stats.linear_model_bytes_saved += 6;
            }
          }
        }
      }

      // Calculate Outlier Storage Overhead
      if outlier_num > 0 {
        let index_bytes = Seg::ef_len_bytes(outlier_num);
        let header = unsafe { crate::ftl::seg::GroupHeader::from_bytes(payload) };
        let outlier_bw = header.outlier_bw() as usize;
        let residual_bits = outlier_num * outlier_bw;
        // Ceiling division for bytes
        let residual_bytes = residual_bits.div_ceil(8);
        stats.outlier_bytes += index_bytes + residual_bytes;
      }
    }
    stats
  }
}
