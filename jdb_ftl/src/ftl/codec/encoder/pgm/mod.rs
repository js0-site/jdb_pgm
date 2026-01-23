use crate::ftl::{bg::PayloadChunk, frame::Head, seg::GroupHeader};

mod assemble;
mod parse;
mod plan;
mod types;

/// PGM Mode encoding with Residual-Patch (PFOR) support.
pub fn encode_pgm(
  group_ppas: &[u64],
  dirty_map: &[bool],
  old_pgm_payload: Option<&[u8]>,
  epsilon: usize,
) -> (Head, Vec<PayloadChunk>) {
  let n = group_ppas.len();
  let mut head = Head::new();

  // 1. Parse Old Structure
  let (old_segs_info, old_outliers_idxs) = parse::parse_old_structure(old_pgm_payload, n);

  // 2. Plan (Greedy with Reuse)
  let (plan, outliers) = plan::generate_plan(
    group_ppas,
    dirty_map,
    epsilon,
    &old_segs_info,
    &old_outliers_idxs,
    4, // skip_threshold
  );

  // 3. Assemble
  // 4. Final Fallback Check (size > n*8) is done in assemble?
  // The original code did fallback check at step 4.
  // Let's implement fallback check here on the result of assemble.

  let (mut encoded_head, chunks) =
    assemble::assemble(n, group_ppas, plan, outliers, &old_segs_info);

  // Calculate total size
  let mut total_bytes = 0;
  for chunk in &chunks {
    match chunk {
      PayloadChunk::New(data) => total_bytes += data.len(),
      PayloadChunk::Reuse { len, .. } => total_bytes += *len as usize,
    }
  }

  if total_bytes > n * 8 {
    // Mode 0: Raw
    let mut raw_payload = Vec::with_capacity(GroupHeader::SIZE + n * 8);
    let raw_header = GroupHeader::new(0, 0, 0, 0);
    raw_payload.extend_from_slice(&raw_header.0.to_le_bytes());
    for &pba in group_ppas {
      raw_payload.extend_from_slice(&pba.to_le_bytes());
    }
    head.set_seg_num(0);
    return (head, vec![PayloadChunk::New(raw_payload)]);
  }

  encoded_head.set_seg_num(encoded_head.seg_num()); // ? assemble sets it.
  (encoded_head, chunks)
}
