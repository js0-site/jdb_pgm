/// A high-performance bit-level pusher for residuals.
/// 高性能位级残留数据写入器。
pub struct BitWriter {
  /// Internal byte buffer. 内部字节缓冲区。
  pub data: Vec<u8>,
  /// Current bit accumulator. 当前位累加器。
  current: u128,
  /// Current bit offset in the accumulator. 累加器中当前的位偏移。
  offset: u8,
}

impl BitWriter {
  /// Create a new BitWriter with specified initial capacity.
  /// 创建具有指定初始容量的新 BitWriter。
  pub fn new(capacity: usize) -> Self {
    Self {
      data: Vec::with_capacity(capacity / 2),
      current: 0,
      offset: 0,
    }
  }

  /// Write a value with specified bit width.
  /// 写入指定位宽的值。
  #[inline(always)]
  pub fn write(&mut self, val: u64, bits: u8) {
    if bits == 0 {
      return;
    }
    let mask = (1u128 << bits) - 1;
    let val = (val as u128) & mask;
    self.current |= val << self.offset;
    self.offset += bits;

    // Flush completed bytes from accumulator.
    while self.offset >= 8 {
      self.data.push(self.current as u8);
      self.current >>= 8;
      self.offset -= 8;
    }
  }

  /// Align the current stream to the next byte boundary.
  /// 将当前流对齐到下一个字节边界。
  #[inline(always)]
  pub fn byte_align(&mut self) {
    if self.offset > 0 {
      self.data.push(self.current as u8);
      self.current = 0;
      self.offset = 0;
    }
  }

  /// Append raw bytes directly, ensuring byte alignment first.
  /// 直接追加原始字节，首先确保字节对齐。
  #[inline(always)]
  pub fn append_bytes(&mut self, bytes: &[u8]) {
    self.byte_align();
    self.data.extend_from_slice(bytes);
  }

  /// Finalize the bit stream and pad for SIMD safety.
  /// 结束位流并填充以保证 SIMD 安全。
  pub fn finish(mut self) -> Vec<u8> {
    self.byte_align();
    // 16-byte padding for aligned SIMD loads.
    self.data.extend_from_slice(&[0u8; 16]);
    self.data
  }

  /// Finalize with minimal padding (8 bytes for 64-bit safe reads).
  /// 使用最小填充结束（8 字节用于安全的 64 位读取）。
  pub fn finish_minimal(mut self) -> Vec<u8> {
    self.byte_align();
    // 8-byte padding for safe u64 unaligned reads.
    self.data.extend_from_slice(&[0u8; 8]);
    self.data
  }

  /// Get the total number of bits written so far.
  /// 获取目前为止写入的总位数。
  #[inline(always)]
  pub fn total_bits(&self) -> usize {
    self.data.len() * 8 + self.offset as usize
  }

  /// Clear the writer for reuse.
  pub fn clear(&mut self) {
    self.data.clear();
    self.current = 0;
    self.offset = 0;
  }
}
