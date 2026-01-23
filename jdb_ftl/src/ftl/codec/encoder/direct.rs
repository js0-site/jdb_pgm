use crate::ftl::{bg::PayloadChunk, codec::bit_writer::BitWriter, frame::Head};

/// Direct Mode optimization for sparse/small groups.
pub fn encode_direct(group_ppas: &[u64]) -> (Head, Vec<PayloadChunk>) {
  let n = group_ppas.len();
  let mut header = Head::new();

  let min_val = group_ppas.iter().copied().min().unwrap_or(0);
  let max_val = group_ppas.iter().copied().max().unwrap_or(0);
  let diff = max_val - min_val;

  // Width based on delta, not absolute value.
  let width = if diff == 0 {
    0
  } else {
    (64 - diff.leading_zeros()) as u8
  };

  // Calculate how many prefix bytes are shared (0-8).
  let base_len = if min_val == 0 {
    0
  } else {
    (64 - min_val.leading_zeros()).div_ceil(8) as u8
  };

  header.set_direct(true);
  header.set_count(n as u8);
  header.set_width(width);
  header.set_base_len(base_len);

  let mut payload = Vec::with_capacity(base_len as usize + (n * width as usize) / 8 + 16);

  // Write shared base (min_val).
  let base_bytes = min_val.to_le_bytes();
  payload.extend_from_slice(&base_bytes[..base_len as usize]);

  if width > 0 {
    let mut writer = BitWriter::new(n * width as usize);
    for &val in group_ppas {
      writer.write(val - min_val, width);
    }
    let bytes = writer.finish_minimal();
    payload.extend_from_slice(&bytes);
  }

  (header, vec![PayloadChunk::New(payload)])
}
