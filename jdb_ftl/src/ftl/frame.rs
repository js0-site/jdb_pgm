#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Head {
  /// Packed field:
  /// - PGM Mode: bits 0-13 = seg_num, bit 14 = rsvd, bit 15 = is_direct (0)
  /// - Direct Mode: [15] is_direct (1), [11-14] count, [5-10] width, [2-4] base_len, [0-1] rsvd
  packed: u16,
}

impl Head {
  pub const SIZE: usize = 2;

  const FLAG_DIRECT: u16 = 1 << 15;
  const SEG_NUM_MASK: u16 = 0x3FFF;

  #[inline(always)]
  pub fn new() -> Self {
    Self { packed: 0 }
  }

  /// Create header with segment count.
  /// 创建带线段数的头部。
  #[inline(always)]
  pub fn with_seg_num(seg_num: u16) -> Self {
    debug_assert!(seg_num <= Self::SEG_NUM_MASK);
    Self { packed: seg_num }
  }

  /// Get segment count.
  /// 获取线段数。
  #[inline(always)]
  pub fn seg_num(&self) -> u16 {
    self.packed & Self::SEG_NUM_MASK
  }

  /// Set segment count.
  /// 设置线段数。
  #[inline(always)]
  pub fn set_seg_num(&mut self, num: u16) {
    debug_assert!(num <= Self::SEG_NUM_MASK);
    self.packed = (self.packed & Self::FLAG_DIRECT) | (num & Self::SEG_NUM_MASK);
  }

  /// Check if direct mode (no PGM).
  /// 检查是否为直接模式（无 PGM）。
  #[inline(always)]
  pub fn is_direct(&self) -> bool {
    self.packed & Self::FLAG_DIRECT != 0
  }

  /// Set direct mode flag.
  /// 设置直接模式标志。
  #[inline(always)]
  pub fn set_direct(&mut self, direct: bool) {
    if direct {
      self.packed |= Self::FLAG_DIRECT;
    } else {
      self.packed &= !Self::FLAG_DIRECT;
    }
  }

  /// Check if header is empty (no segments).
  /// 检查头部是否为空（无线段）。
  /// Note: Only applicable if not in Direct Mode.
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    if self.is_direct() {
      self.count() == 0
    } else {
      false // PGM groups are never purely empty in this context (handled by V2 logic) or seg_num check if we revert to that.
      // Actually, in the new PGM (V2) mode, is_empty on the Head struct itself is less relevant because metadata is in the payload GroupHeader.
      // But let's keep the logic consistent: if it's not direct, it's PGM.
      // Original code said "V2 groups are never purely empty".
    }
  }

  // --- Direct Mode Accessors ---

  #[inline(always)]
  pub fn count(&self) -> u8 {
    ((self.packed >> 11) & 0xF) as u8
  }

  #[inline(always)]
  pub fn set_count(&mut self, n: u8) {
    self.packed = (self.packed & !(0xF << 11)) | ((n as u16 & 0xF) << 11);
  }

  #[inline(always)]
  pub fn width(&self) -> u8 {
    ((self.packed >> 5) & 0x3F) as u8
  }

  #[inline(always)]
  pub fn set_width(&mut self, w: u8) {
    self.packed = (self.packed & !(0x3F << 5)) | ((w as u16 & 0x3F) << 5);
  }

  #[inline(always)]
  pub fn base_len(&self) -> u8 {
    ((self.packed >> 2) & 0x7) as u8
  }

  #[inline(always)]
  pub fn set_base_len(&mut self, len: u8) {
    self.packed = (self.packed & !(0x7 << 2)) | ((len as u16 & 0x7) << 2);
  }

  /// Cast a byte slice to a Head reference.
  /// 将字节切片转换为 Head 引用。
  ///
  /// # Safety
  /// The caller must ensure that the byte slice is at least 2 bytes.
  #[inline(always)]
  pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
    let val = unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const u16) };
    Self { packed: val }
  }

  /// Get the header bytes.
  /// 获取头部的字节视图。
  #[inline(always)]
  pub fn as_bytes(&self) -> &[u8; 2] {
    unsafe { std::mem::transmute(self) }
  }
}



/// Represents a physical storage frame (Used in higher-level APIs).
#[derive(Debug, Clone)]
pub struct Frame {
  pub header: Head,
  pub payload: Vec<u8>,
}

impl Default for Frame {
  fn default() -> Self {
    Self::new()
  }
}

impl Frame {
  pub fn new() -> Self {
    Self {
      header: Head::new(),
      payload: Vec::new(),
    }
  }
}
