use std::{cell::UnsafeCell, sync::Arc};

/// L1 Storage Layers.
/// L1 存储层。
/// A consolidated Group entry in L1.
/// L1 中的合并组条目。
///
/// Optimized to 16 bytes: only a Box pointer to the combined Header+Payload.
/// 优化至 16 字节：仅包含指向合并 Header+Payload 的 Box 指针。
#[repr(transparent)]
pub struct Group {
  /// Combined metadata header and compressed data storage.
  /// 合并的元数据头部和压缩数据存储。
  /// Layout: [Head (16B)] | [Segs] | [Exceptions] | [Residuals]
  pub storage: Box<[u8]>,
}

/// L1 Storage Layers.
/// L1 存储层。
pub struct L1 {
  /// Array of groups, each 16 bytes.
  /// 组数组，每个 16 字节。
  pub groups: Box<[Group]>,
}

/// A wrapper around UnsafeCell that is Sync.
#[repr(transparent)]
pub struct InternalSyncCell<T>(UnsafeCell<T>);

impl<T> InternalSyncCell<T> {
  pub fn new(value: T) -> Self {
    Self(UnsafeCell::new(value))
  }
  pub fn get(&self) -> *mut T {
    self.0.get()
  }
}

unsafe impl<T: Send> Send for InternalSyncCell<T> {}
unsafe impl<T: Send> Sync for InternalSyncCell<T> {}

/// Thread-safe wrapper for L1 using UnsafeCell.
pub struct SharedL1 {
  inner: Arc<InternalSyncCell<L1>>,
}

impl Clone for SharedL1 {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

unsafe impl Send for SharedL1 {}
unsafe impl Sync for SharedL1 {}

impl SharedL1 {
  pub fn new(capacity_lba: usize, group_size: usize) -> Self {
    let num_groups = capacity_lba.div_ceil(group_size);
    let mut groups = Vec::with_capacity(num_groups);
    for _ in 0..num_groups {
      groups.push(Group {
        storage: Vec::new().into_boxed_slice(),
      });
    }
    let l1 = L1 {
      groups: groups.into_boxed_slice(),
    };
    Self {
      inner: Arc::new(InternalSyncCell::new(l1)),
    }
  }

  /// Get a raw pointer to the underlying L1.
  /// 获取底层 L1 的裸指针。
  ///
  /// # Safety
  /// This pointer allows bypassing Rust's borrowing rules.
  /// Caller must ensure correct concurrency (e.g. no overlapping mutable access to same fields).
  #[inline(always)]
  pub fn as_ptr(&self) -> *mut L1 {
    self.inner.get()
  }

  /// Get an immutable reference to the underlying L1.
  /// 获取底层 L1 的不可变引用。
  ///
  /// # Safety
  /// The caller must ensure that no other thread is holding a mutable reference.
  /// 调用者必须确保没有其他线程持有可变引用。
  #[inline(always)]
  pub unsafe fn get_ref(&self) -> &L1 {
    unsafe { &*self.inner.get() }
  }

  /// Get a mutable reference to the underlying L1.
  /// 获取底层 L1 的可变引用。
  ///
  /// # Safety
  /// The caller must ensure exclusive access to the underlying L1 data.
  /// 调用者必须确保对底层 L1 数据的独占访问。
  #[inline(always)]
  #[allow(clippy::mut_from_ref)]
  pub unsafe fn get_mut(&self) -> &mut L1 {
    unsafe { &mut *self.inner.get() }
  }
}
