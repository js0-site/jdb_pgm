use super::{direct::encode_direct, pgm::encode_pgm};
use crate::ftl::{bg::PayloadChunk, frame::Head};

/// Use multi-segment PGM with incremental support to encode a group of PPAs.
/// 使用具有增量支持的多段 PGM 编码一组 PPA。
/// Use multi-segment PGM with incremental support to encode a group of PPAs.
/// 使用具有增量支持的多段 PGM 编码一组 PPA。
pub fn encode(
  group_ppas: &[u64],
  dirty_map: &[bool],
  old_pgm_payload: Option<&[u8]>,
  epsilon: usize,
) -> (Head, Vec<PayloadChunk>) {
  let n = group_ppas.len();

  // 1. Empty Mode
  if n == 0 {
    return (Head::new(), Vec::new());
  }

  // 2. Direct Mode optimization for sparse/small groups.
  // 对于稀疏/小组的直接模式优化。
  if n <= 8 {
    return encode_direct(group_ppas);
  }

  // 3. PGM Mode (Piecewise Linear Regression)
  // PGM 模式（分段线性回归）
  encode_pgm(group_ppas, dirty_map, old_pgm_payload, epsilon)
}
