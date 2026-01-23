use super::types::{OldSegInfo, Plan};
use crate::ftl::{
  bg::PayloadChunk,
  codec::{bit_width, bit_writer::BitWriter, ef, zigzag_encode},
  frame::Head,
  seg::{GroupHeader, Seg},
};

pub fn assemble(
  n: usize,
  group_ppas: &[u64],
  plan: Vec<Plan>,
  outliers: Vec<(u16, u64)>,
  old_segs_info: &[OldSegInfo],
) -> (Head, Vec<PayloadChunk>) {
  let mut head = Head::new();

  // Calculate Outlier BW
  let mut max_outlier_res = 0u64;
  let mut outlier_data = Vec::with_capacity(outliers.len());

  // Need to map outliers to Segments to calculate prediction
  {
    let mut seg_base_map = Vec::new();
    let mut current_start = 0;
    for p in &plan {
      match p {
        Plan::Reuse { old_idx } => {
          let old = &old_segs_info[*old_idx];
          let len = (old.end - old.start) as usize;
          seg_base_map.push((current_start, old.seg.base(), old.seg.slope()));
          current_start += len;
        }
        Plan::New { fit, len, .. } => {
          seg_base_map.push((current_start, fit.base, fit.slope));
          current_start += *len as usize;
        }
      }
    }

    let mut seg_cursor = 0;
    for (lba_off, pba) in &outliers {
      // Advance seg_cursor to find the correct segment for this outlier.
      // Since outliers and seg_base_map are sorted by LBA, we can move forward.
      // 我们通过前向移动游标来匹配段，利用了数据的有序性。
      while seg_cursor + 1 < seg_base_map.len()
        && seg_base_map[seg_cursor + 1].0 <= *lba_off as usize
      {
        seg_cursor += 1;
      }

      let (start, base, slope) = seg_base_map[seg_cursor];

      let pred = base
        .wrapping_add(((*lba_off as i64 - start as i64).wrapping_mul(slope as i64) >> 24) as u64);
      let zz = zigzag_encode(*pba as i64 - pred as i64);
      max_outlier_res = max_outlier_res.max(zz);
      outlier_data.push((*lba_off, zz));
    }
  }
  let outlier_bw = bit_width(max_outlier_res);

  // Encode Indices
  let outlier_ef = ef::encode(&outliers.iter().map(|(o, _)| *o).collect::<Vec<_>>(), n);
  // Seg start indices
  let mut seg_starts = Vec::new();
  let mut curr = 0;
  for p in &plan {
    seg_starts.push(curr as u16);
    match p {
      Plan::Reuse { old_idx } => {
        curr += (old_segs_info[*old_idx].end - old_segs_info[*old_idx].start) as usize
      }
      Plan::New { len, .. } => curr += *len as usize,
    }
  }
  let seg_ef = ef::encode(&seg_starts, n);

  let outlier_idx_size = Seg::ef_len_bytes(outliers.len());
  let seg_idx_size = Seg::ef_len_bytes(plan.len());
  let seg_table_size = plan.len() * Seg::METADATA_SIZE;

  // Calculate offsets
  let mut current_byte_offset =
    (GroupHeader::SIZE + outlier_idx_size + seg_idx_size + seg_table_size) as u32;
  // Align to byte?
  if !current_byte_offset.is_multiple_of(2) {
    current_byte_offset += 1; // Padded for u16 alignment of Seg table
  }

  let mut writer = BitWriter::new(n / 4);
  let mut chunks = Vec::new();
  let mut aligned_meta = Vec::new();

  let g_header = GroupHeader::new(1, plan.len() as u16, outliers.len() as u16, outlier_bw);
  aligned_meta.extend_from_slice(&g_header.0.to_le_bytes());
  aligned_meta.extend_from_slice(&outlier_ef);
  aligned_meta.extend_from_slice(&seg_ef);
  if aligned_meta.len() % 2 != 0 {
    aligned_meta.push(0);
  }

  // Seg Table
  let mut seg_descriptors = Vec::new();

  // We'll handle everything as Byte aligned for simplicity and Reuse.
  let mut running_byte_offset = current_byte_offset as usize;

  for p in &plan {
    match p {
      Plan::Reuse { old_idx } => {
        let old = &old_segs_info[*old_idx];
        let bw = old.seg.bit_width();
        let b_off = if bw == 0 {
          0
        } else {
          running_byte_offset as u32
        };
        let seg = Seg::new(old.seg.base(), old.seg.slope(), b_off, bw);
        seg_descriptors.push(seg);
        if bw > 0 {
          running_byte_offset += old.len_bytes;
        }
      }
      Plan::New { fit, len, max_res } => {
        let bw = if *max_res == 0 {
          0
        } else {
          bit_width(*max_res)
        };
        let b_off = if bw == 0 {
          0
        } else {
          running_byte_offset as u32
        };
        let seg = Seg::new(fit.base, fit.slope, b_off, bw);
        seg_descriptors.push(seg);
        if bw > 0 {
          let bits = *len as usize * bw as usize;
          let bytes = bits.div_ceil(8);
          running_byte_offset += bytes;
        }
      }
    }
  }

  // Write Seg Table to meta
  for seg in &seg_descriptors {
    unsafe {
      let ptr = seg as *const Seg as *const u8;
      aligned_meta.extend_from_slice(std::slice::from_raw_parts(ptr, Seg::METADATA_SIZE));
    }
  }

  // Now we have the Meta chunk.
  chunks.push(PayloadChunk::New(aligned_meta));

  // Now emit the residual chunks
  for (i, p) in plan.iter().enumerate() {
    match p {
      Plan::Reuse { old_idx } => {
        let old = &old_segs_info[*old_idx];
        if old.seg.bit_width() > 0 {
          // Push Reuse Chunk
          chunks.push(PayloadChunk::Reuse {
            offset: old.byte_offset as u32,
            len: old.len_bytes as u32,
          });
        }
      }
      Plan::New { fit, len, max_res } => {
        let bw = if *max_res == 0 {
          0
        } else {
          bit_width(*max_res)
        };
        if bw > 0 {
          let start_idx = seg_starts[i] as usize;
          let chunk = &group_ppas[start_idx..start_idx + *len as usize];

          writer.clear(); // Reset writer for this block
          for (j, &v) in chunk.iter().enumerate() {
            let p = fit
              .base
              .wrapping_add(((j as i64).wrapping_mul(fit.slope as i64) >> 24) as u64);
            writer.write(v.wrapping_sub(p), bw);
          }
          writer.byte_align();
          chunks.push(PayloadChunk::New(writer.data.clone()));
        }
      }
    }
  }

  // Outlier Residuals
  if outlier_bw > 0 {
    writer.clear();
    for (_, zz_res) in outlier_data {
      writer.write(zz_res, outlier_bw);
    }
    chunks.push(PayloadChunk::New(writer.finish()));
  }

  head.set_seg_num(plan.len() as u16);
  (head, chunks)
}
