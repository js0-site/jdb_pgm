/// Skip table sampling interval (elements per skip entry).
/// 跳表采样间隔（每个跳表项对应的元素数）。
pub const SKIP_INTERVAL: usize = 64;

/// Helper to read bits from a byte slice.
/// 帮助函数：从字节切片中读取位。
#[inline(always)]
pub fn read_bits_u64_at(data: &[u8], base_offset: usize, bit_idx: usize, len: usize) -> u64 {
  if len == 0 {
    return 0;
  }

  let byte_idx = base_offset + (bit_idx / 8);
  let bit_offset = bit_idx % 8;

  // Fast path: 4 bytes (u32) is enough for max len=16, but we might need more for u64 support later.
  // Assuming we read up to 64 bits potentially in future, but for now u16 implies len <= 16.
  // However, for generic support, let's keep it safe.

  // Safety: ensure we don't read past end
  if byte_idx + 8 <= data.len() {
    let val = unsafe { (data.as_ptr().add(byte_idx) as *const u64).read_unaligned() };
    // Little endian read
    let val_le = u64::from_le(val);
    return (val_le >> bit_offset) & ((1u64 << len) - 1);
  }

  // Slow path
  let mut w = 0u64;
  let end = (byte_idx + 8).min(data.len());
  for i in 0..(end - byte_idx) {
    unsafe {
      w |= (*data.get_unchecked(byte_idx + i) as u64) << (i * 8);
    }
  }

  (w >> bit_offset) & ((1u64 << len) - 1)
}

/// Helper to select the nth set bit index in a byte.
/// 帮助函数：选择字节中第 n 个置位比特的索引。
#[inline(always)]
pub fn select_bit_in_byte(mut w: u8, needed: usize) -> usize {
  // 8 iterations max, fast linear scan or loop
  // 最多8次迭代，快速线性扫描或循环
  for _ in 0..needed {
    w &= w.wrapping_sub(1);
  }
  w.trailing_zeros() as usize
}

/// Helper to select the nth set bit index in a u64.
/// 帮助函数：选择 u64 中第 n 个置位比特的索引。
#[inline(always)]
pub fn select_bit_in_u64(mut w: u64, mut needed: usize) -> usize {
  // Could use PDEP/TZCNT loops or Broadword selection, but simple loop is fine for avg interval 32
  // 可以使用 PDEP/TZCNT 循环或 Broadword 选择，但简单的循环对于平均间隔 32 来说已经足够
  while needed > 0 {
    w &= w.wrapping_sub(1); // clear LSB
    needed -= 1;
  }
  w.trailing_zeros() as usize
}

/// Load u64 safe from potentially partial buffer tail.
/// 安全地从潜在的缓冲区尾部加载 u64。
#[inline(always)]
pub fn load_u64_safe(data: &[u8], byte_idx: usize) -> u64 {
  let mut w = 0u64;
  let remaining = data.len().saturating_sub(byte_idx).min(8);

  // Optimized manual loop safe for compiler unrolling
  // 编译器可展开的优化手动循环
  for i in 0..remaining {
    unsafe {
      w |= (*data.get_unchecked(byte_idx + i) as u64) << (i * 8);
    }
  }
  w
}
