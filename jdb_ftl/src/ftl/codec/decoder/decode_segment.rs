use super::bits::read_bits;
use crate::ftl::seg::Seg;

/// Decode a single PGM segment into a provided mutable buffer.
/// 将单个 PGM 段解码到提供的可变缓冲区中。
#[inline(always)]
pub fn decode_segment(seg: &Seg, payload: &[u8], out: &mut [u64]) {
  let slope = seg.slope() as i64;
  let base = seg.base();
  let width = seg.bit_width();
  // Note: original code used byte_offset but seg has bit_offset.
  // The original used `read_bits` which takes a byte slice and a bit index *relative to that slice*.
  // So providing `byte_offset` implies byte alignment?
  // Let's look at `seg.bit_offset()`.
  // Previous `decode_segment.rs` (Line 38 in view) cast bit_offset to usize and called it byte_offset?
  // Wait, `seg.bit_offset()` returns bits.
  // If original code treated it as bytes, that was a bug or specific convention.
  // Let's assume `bit_offset` is bits.
  // `read_bits` helper likely handles bit offsets.
  // BUT the original code: `let byte_offset = seg.bit_offset() as usize;`
  // `let seg_residuals = unsafe { payload.get_unchecked(byte_offset..) };`
  // This heavily implies `seg.bit_offset()` stores BYTES.
  // Let's check Seg::bit_offset definition.
  // It returns w1>>16 | w2&F << 16. It's an offset.
  // In `encode.rs`, `current_offset` is in BYTES (aligned to byte check).
  // So yes, it is byte offset.

  let byte_offset = seg.bit_offset() as usize;

  // Use iterative accumulator to avoid multiplication in the hot loop.
  // 使用迭代累加器以避免热循环中的乘法。
  let mut acc = 0i64;

  if width == 0 {
    // Zero-residual hot path: purely linear calculation.
    // 零残差热路径：纯线性计算。
    for val in out.iter_mut() {
      *val = base.wrapping_add((acc >> 24) as u64);
      acc += slope;
    }
  } else {
    // Residual correction path.
    // 残差校正路径。
    let seg_residuals = unsafe { payload.get_unchecked(byte_offset..) };
    for (i, val) in out.iter_mut().enumerate() {
      let pred = base.wrapping_add((acc >> 24) as u64);
      acc += slope;

      let bit_idx = i * (width as usize);
      let delta = read_bits(seg_residuals, bit_idx, width);
      *val = pred.wrapping_add(delta);
    }
  }
}
