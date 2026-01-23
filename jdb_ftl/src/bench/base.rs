use crate::FtlTrait;

/// Simple baseline FlatMap implementation for benchmarking comparisons.
/// 用于驱动基准测试对比的简单基准 FlatMap 实现。
#[derive(Clone)]
pub struct Base {
  table: Box<[u64]>,
}

impl Base {
  /// Create a new FlatMap with specified capacity.
  /// 创建具有指定容量的新 FlatMap。
  pub fn new(cap: usize) -> Self {
    Self {
      table: vec![u64::MAX; cap].into_boxed_slice(),
    }
  }
}

impl FtlTrait for Base {
  fn new(size: u64) -> Self {
    Self::new(size as usize)
  }

  #[inline(always)]
  fn get(&self, lba: u64) -> Option<u64> {
    let idx = lba as usize;
    if idx >= self.table.len() {
      return None;
    }
    // SAFETY: Bounds checked above.
    let pba = unsafe { *self.table.get_unchecked(idx) };
    if pba == u64::MAX { None } else { Some(pba) }
  }

  #[inline(always)]
  fn set(&mut self, lba: u64, pba: u64) {
    let idx = lba as usize;
    if idx < self.table.len() {
      // SAFETY: Bounds checked above.
      unsafe {
        *self.table.get_unchecked_mut(idx) = pba;
      }
    }
  }

  #[inline(always)]
  #[cfg(feature = "stats")]
  fn mem(&mut self) -> usize {
    self.table.len() * 8 + std::mem::size_of::<Self>()
  }
}
