use crate::ftl::codec::{BitWriter, ef::util::SKIP_INTERVAL};

/// Calculate byte length of encoded EF block including skip table.
pub fn byte_len(n: usize, u_bound: usize) -> usize {
  if n == 0 {
    return 1;
  }
  let l = if u_bound > n {
    (u_bound as f64 / n as f64).log2().floor() as usize
  } else {
    0
  };

  // Header: 3 bytes (1 byte L + 2 bytes UpperLen)
  let mut size = 3;

  // Upper
  let upper_val_bound = u_bound >> l;
  let upper_len_bits = n + upper_val_bound + 1;
  let upper_bytes = upper_len_bits.div_ceil(8);
  size += upper_bytes;

  // Lower
  let lower_len_bits = n * l;
  let lower_bytes = lower_len_bits.div_ceil(8);
  size += lower_bytes;

  // Skip table: each entry is 4 bytes (u16 bit_pos + u16 high_val)
  let skip_count = n.div_ceil(SKIP_INTERVAL);
  size += skip_count * 4;

  size
}

/// Encoder for customized EF with skip table (u16 version).
/// 带跳表的自定义 EF 编码器。
pub fn encode(data: &[u16], u_bound: usize) -> Vec<u8> {
  let n = data.len();
  if n == 0 {
    return vec![0];
  }

  // 1. Calculate L
  let l = if u_bound > n {
    (u_bound as f64 / n as f64).log2().floor() as usize
  } else {
    0
  };

  // 2. Build Upper/Lower and track skip points
  let mut upper_bits = BitWriter::new(n * 2 / 8);
  let mut lower_bits = BitWriter::new(n * l / 8);

  let low_mask = (1u64 << l) - 1;
  let skip_count = n.div_ceil(SKIP_INTERVAL);
  let mut skip_table: Vec<(u16, u16)> = Vec::with_capacity(skip_count);

  let mut upper_bit_pos: usize = 0;

  for (i, &val) in data.iter().enumerate() {
    if i % SKIP_INTERVAL == 0 {
      let prev_h = if i == 0 { 0 } else { (data[i - 1] as u64) >> l };
      skip_table.push((upper_bit_pos as u16, prev_h as u16));
    }

    // Lower
    let low = (val as u64) & low_mask;
    lower_bits.write(low, l as u8);

    // Upper
    let h = (val as u64) >> l;
    let prev_h = if i == 0 { 0 } else { (data[i - 1] as u64) >> l };
    let gap = h.saturating_sub(prev_h);

    let mut rem = gap;
    while rem > 0 {
      let chunk = rem.min(64);
      upper_bits.write(0, chunk as u8);
      rem -= chunk;
      upper_bit_pos += chunk as usize;
    }
    upper_bits.write(1, 1);
    upper_bit_pos += 1;
  }

  // Flush
  upper_bits.byte_align();
  let upper_len_bytes = upper_bits.data.len();

  // Allocate output
  let lower_bytes = lower_bits.total_bits().div_ceil(8);
  let mut out = Vec::with_capacity(2 + upper_len_bytes + lower_bytes + skip_count * 4);

  // Header
  // [L: u8] [UpperLen: u16]
  out.push((l & 0x0F) as u8);
  out.extend_from_slice(&(upper_len_bytes as u16).to_le_bytes()); // 2 bytes

  out.extend_from_slice(&upper_bits.data);

  lower_bits.byte_align();
  out.extend_from_slice(&lower_bits.data);

  // Append skip table (LE)
  for (bit_pos, high_val) in skip_table {
    out.extend_from_slice(&bit_pos.to_le_bytes());
    out.extend_from_slice(&high_val.to_le_bytes());
  }

  out
}
