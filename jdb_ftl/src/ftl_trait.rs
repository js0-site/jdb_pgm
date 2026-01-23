/// FTL mapping interface
/// FTL 映射接口
pub trait FtlTrait {
  /// Create new FTL instance with given capacity (LBAs)
  /// 创建给定容量（LBA 计数）的 FTL 实例
  fn new(size: u64) -> Self;

  /// Get mapping: returns PBA, or None if not mapped
  /// 获取映射：返回 PBA，如果未映射返回 None
  fn get(&self, lba: u64) -> Option<u64>;

  /// Set mapping: LBA -> PBA
  /// 建立/更新映射：LBA -> PBA
  fn set(&mut self, lba: u64, pba: u64);

  /// Get current memory occupancy (Bytes)
  /// 统计内存占用 (Bytes)
  /// This is async to allow implementations to sync pending operations first
  #[cfg(feature = "stats")]
  fn mem(&mut self) -> usize;
}
