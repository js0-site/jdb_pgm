#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GroupHeader(pub u32);

impl GroupHeader {
  pub const SIZE: usize = 4;

  #[inline(always)]
  pub fn new(mode: u8, seg_count: u16, outlier_count: u16, outlier_bw: u8) -> Self {
    let val = (mode as u32 & 0x3)
      | ((seg_count as u32 & 0x3FF) << 2) // 10 bits for segs (0-1023)
      | ((outlier_count as u32 & 0xFFF) << 12) // 12 bits for outliers (0-4095)
      | ((outlier_bw as u32 & 0x3F) << 24); // 6 bits for bw (0-63)
    Self(val)
  }

  #[inline(always)]
  pub fn mode(&self) -> u8 {
    (self.0 & 0x3) as u8
  }
  #[inline(always)]
  pub fn seg_count(&self) -> u16 {
    ((self.0 >> 2) & 0x3FF) as u16
  }
  #[inline(always)]
  pub fn outlier_count(&self) -> u16 {
    ((self.0 >> 12) & 0xFFF) as u16
  }
  #[inline(always)]
  pub fn outlier_bw(&self) -> u8 {
    ((self.0 >> 24) & 0x3F) as u8
  }
  #[inline(always)]
  pub fn flags(&self) -> u8 {
    ((self.0 >> 30) & 0x3) as u8
  }

  /// # Safety
  /// The provided byte slice must contain at least 4 bytes and be readable as a u32.
  #[inline(always)]
  pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
    unsafe { Self(std::ptr::read_unaligned(bytes.as_ptr() as *const u32)) }
  }
}

/// Compact Seg Descriptor (12 Bytes).
/// 紧凑的段描述符（12 字节）。
/// Using u16 fields to relax alignment requirement to 2 bytes.
/// allowing it to follow StartIdxs (u16 array) without padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // Default alignment for u16 is 2.
pub struct Seg {
  // Word 0: Base[0..32]
  // Splitting u32 into u16 pairs to relax alignment to 2 bytes.
  // 将 u32 拆分为 u16 对，将对齐要求降低到 2 字节。
  pub w0_lo: u16,
  pub w0_hi: u16,

  // Word 1: Base[32..48] | Offset[0..16]
  pub w1_lo: u16,
  pub w1_hi: u16,

  // Word 2: Offset[16..20] | Slope[0..22] | Width[0..6]
  pub w2_lo: u16,
  pub w2_hi: u16,
}

impl Seg {
  pub const METADATA_SIZE: usize = 12;

  #[inline(always)]
  pub fn new(base: u64, slope: i32, bit_offset: u32, bit_width: u8) -> Self {
    let base = base & 0xFFFF_FFFF_FFFF; // 48 bits
    let offset = bit_offset;

    // W0
    let w0 = base as u32;
    // W1
    let base_hi = (base >> 32) as u32; // 16 bits
    let off_lo = offset & 0xFFFF; // 16 bits
    let w1 = base_hi | (off_lo << 16);

    // W2
    let off_hi = (offset >> 16) & 0xF; // 4 bits
    let s_enc = (slope & 0x3F_FFFF) as u32; // 22 bits mask
    let w_enc = (bit_width & 0x3F) as u32; // 6 bits mask
    let w2 = off_hi | (s_enc << 4) | (w_enc << 26);

    Self {
      w0_lo: w0 as u16,
      w0_hi: (w0 >> 16) as u16,
      w1_lo: w1 as u16,
      w1_hi: (w1 >> 16) as u16,
      w2_lo: w2 as u16,
      w2_hi: (w2 >> 16) as u16,
    }
  }

  #[inline(always)]
  fn w0(&self) -> u32 {
    (self.w0_lo as u32) | ((self.w0_hi as u32) << 16)
  }

  #[inline(always)]
  fn w1(&self) -> u32 {
    (self.w1_lo as u32) | ((self.w1_hi as u32) << 16)
  }

  #[inline(always)]
  fn w2(&self) -> u32 {
    (self.w2_lo as u32) | ((self.w2_hi as u32) << 16)
  }

  #[inline(always)]
  pub fn base(&self) -> u64 {
    let lo = self.w0() as u64;
    let hi = (self.w1() & 0xFFFF) as u64;
    lo | (hi << 32)
  }

  #[inline(always)]
  pub fn slope(&self) -> i32 {
    // Extract 22 bits from bit 4
    let raw = (self.w2() >> 4) & 0x3F_FFFF;
    // Sign extend 22 bits to 32 bits
    if raw & 0x20_0000 != 0 {
      (raw | 0xFFC0_0000) as i32
    } else {
      raw as i32
    }
  }

  #[inline(always)]
  pub fn bit_offset(&self) -> u32 {
    let lo = (self.w1() >> 16) & 0xFFFF;
    let hi = self.w2() & 0xF;
    lo | (hi << 16)
  }

  #[inline(always)]
  pub fn bit_width(&self) -> u8 {
    ((self.w2() >> 26) & 0x3F) as u8
  }

  /// Estimate the size of the EF-encoded blocks.
  #[inline(always)]
  pub fn ef_len_bytes(num: usize) -> usize {
    match num {
      0 => 0,
      _ => crate::ftl::codec::ef::byte_len(num, 4096),
    }
  }

  pub fn view_outlier_ef(
    payload: &[u8],
    outlier_num: usize,
  ) -> crate::ftl::codec::ef::EfViewU16<'_> {
    // Outlier Index starts after GroupHeader (4 bytes)
    let start = GroupHeader::SIZE;
    crate::ftl::codec::ef::EfViewU16::new(&payload[start..], outlier_num, 4096)
  }

  pub fn view_seg_ef(
    payload: &[u8],
    outlier_num: usize,
    seg_num: usize,
  ) -> crate::ftl::codec::ef::EfViewU16<'_> {
    let start = GroupHeader::SIZE + Self::ef_len_bytes(outlier_num);
    crate::ftl::codec::ef::EfViewU16::new(&payload[start..], seg_num, 4096)
  }

  #[inline(always)]
  pub fn view_table(payload: &[u8], outlier_num: usize, seg_num: usize) -> &[Self] {
    if seg_num == 0 {
      return &[];
    }
    // Offset = Header + OutlierEF + SegEF
    let offset = GroupHeader::SIZE + Self::ef_len_bytes(outlier_num) + Self::ef_len_bytes(seg_num);
    // Align to 2 bytes for Seg struct (u16 fields)
    let aligned_offset = (offset + 1) & !1;
    unsafe {
      let ptr = payload.as_ptr().add(aligned_offset) as *const Self;
      std::slice::from_raw_parts(ptr, seg_num)
    }
  }
}
