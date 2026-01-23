/// Fast bit-level reader from a byte slice.
/// 从字节切片中快速读取位。
///
/// Uses a 128-bit unaligned peek to minimize branching and memory access for small bit-widths.
/// 使用 128 位非对齐预览，以最小化小位宽时的分支和内存访问。
#[inline(always)]
pub fn read_bits(data: &[u8], bit_idx: usize, len: u8) -> u64 {
  let byte_idx = bit_idx >> 3;
  let bit_offset = (bit_idx & 7) as u8;

  unsafe {
    let ptr = data.as_ptr().add(byte_idx);
    // Optimization: Use u64 load for common small bit-widths (<= 56 bits).
    // This avoids 128-bit operations on 32-bit systems and reduces register pressure.
    // 优化：对于常见的较小位宽（<= 56 位），使用 u64 加载。
    // 这避免了 32 位系统上的 128 位操作并减少了寄存器压力。
    if len <= 56 {
      let val_u64 = std::ptr::read_unaligned(ptr as *const u64);
      let mask = (1u64 << len) - 1;
      (val_u64 >> bit_offset) & mask
    } else {
      // peek up to 16 bytes for 128-bit overlap.
      // 预览最多 16 个字节以进行 128 位重叠读取。
      let val_u128 = std::ptr::read_unaligned(ptr as *const u128);
      let mask = (1u64 << len) - 1;
      ((val_u128 >> bit_offset) as u64) & mask
    }
  }
}
