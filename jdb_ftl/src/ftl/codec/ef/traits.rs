use std::fmt::Debug;

/// Trait defining the primitive type stored in the Elias-Fano sequence (e.g., u16, u32, u64).
/// 定义 Elias-Fano 序列中存储的原始类型（例如 u16、u32、u64）。
pub trait EfPrimitive: Copy + Ord + Debug + Default + 'static {
  /// Convert to u64 for bit manipulation.
  fn as_u64(self) -> u64;

  /// Construct from u64 (truncating if necessary).
  fn from_u64(v: u64) -> Self;

  /// Returns 0xFFFF... equivalent for the type - sentinel value.
  fn sentinel() -> Self;
}

impl EfPrimitive for u16 {
  #[inline(always)]
  fn as_u64(self) -> u64 {
    self as u64
  }

  #[inline(always)]
  fn from_u64(v: u64) -> Self {
    v as u16
  }

  #[inline(always)]
  fn sentinel() -> Self {
    0xFFFF
  }
}

impl EfPrimitive for u32 {
  #[inline(always)]
  fn as_u64(self) -> u64 {
    self as u64
  }

  #[inline(always)]
  fn from_u64(v: u64) -> Self {
    v as u32
  }

  #[inline(always)]
  fn sentinel() -> Self {
    0xFFFF_FFFF
  }
}

impl EfPrimitive for u64 {
  #[inline(always)]
  fn as_u64(self) -> u64 {
    self
  }

  #[inline(always)]
  fn from_u64(v: u64) -> Self {
    v
  }

  #[inline(always)]
  fn sentinel() -> Self {
    0xFFFF_FFFF_FFFF_FFFF
  }
}

/// Trait defining the binary layout of the encoded EF structure.
/// 定义编码 EF 结构的二进制布局的 Trait。
/// This allows different layouts for small types (u16) vs large types (u64).
pub trait EfLayout: 'static + Copy {
  type Primitive: EfPrimitive;

  /// Size of a skip table entry in bytes.
  const SKIP_ENTRY_SIZE: usize; // e.g. 4 for u16, 16 for u64

  /// Sampling interval for skip table.
  const SKIP_INTERVAL: usize;

  /// Read a skip table entry from the raw data at the given byte offset.
  /// Returns (start_bit_pos, start_high_val).
  ///
  /// # Safety
  /// Implementation must ensure simple bounds checks or rely on caller checks.
  /// `data` is the entire blob, `offset` is absolute or relative to skip table start?
  /// Usually inputs: `data` slice starting at skip table entry? Or full slice + absolute offset?
  /// Let's use `full slice` + `offset` for safety/flexibility.
  unsafe fn read_skip_entry(data: &[u8], offset: usize) -> (usize, Self::Primitive);

  /// Write a skip table entry to the output buffer.
  fn write_skip_entry(out: &mut Vec<u8>, bit_pos: usize, high_val: Self::Primitive);
}

/// Default layout for u16 (Compact).
/// u16 的默认布局（紧凑型）。
/// Format: [u16 bit_pos] [u16 high_val] (Little Endian)
#[derive(Debug, Clone, Copy)]
pub struct LayoutU16;

impl EfLayout for LayoutU16 {
  type Primitive = u16;
  const SKIP_ENTRY_SIZE: usize = 4;
  const SKIP_INTERVAL: usize = 64;

  #[inline(always)]
  unsafe fn read_skip_entry(data: &[u8], offset: usize) -> (usize, u16) {
    unsafe {
      let ptr = data.as_ptr().add(offset);
      let bit_pos_bytes = (ptr as *const [u8; 2]).read_unaligned();
      let high_val_bytes = (ptr.add(2) as *const [u8; 2]).read_unaligned();
      (
        u16::from_le_bytes(bit_pos_bytes) as usize,
        u16::from_le_bytes(high_val_bytes),
      )
    }
  }

  #[inline(always)]
  fn write_skip_entry(out: &mut Vec<u8>, bit_pos: usize, high_val: u16) {
    out.extend_from_slice(&(bit_pos as u16).to_le_bytes());
    out.extend_from_slice(&high_val.to_le_bytes());
  }
}
