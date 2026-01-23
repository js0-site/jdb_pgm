use std::sync::Arc;

#[cfg(feature = "stats")]
use crate::ftl::stats::FtlStats;
use crate::{
  Ftl, FtlTrait,
  ftl::{codec, conf::Conf, frame::Head},
};

pub type DefaultFtl = Ftl<crate::ftl::conf::FtlConf>;

impl<C: Conf> FtlTrait for Ftl<C> {
  fn new(size: u64) -> Self {
    Self::new_with_capacity(size as usize)
  }

  #[inline(always)]
  fn get(&self, lba: u64) -> Option<u64> {
    if let Some(&pba) = self.write_buffer.get(&lba) {
      return if pba == u64::MAX { None } else { Some(pba) };
    }

    // Check Flushing Buffers (L0 -> L1 transition)
    // 检查刷新缓冲区 (L0 -> L1 过渡)
    if !self.flushing.is_empty() {
      for buf in self.flushing.iter().rev() {
        if let Some(&pba) = buf.get(&lba) {
          return if pba == u64::MAX { None } else { Some(pba) };
        }
      }
    }

    let n = C::GROUP_SIZE;
    let group_idx = (lba / n as u64) as usize;

    if group_idx >= self.num_groups {
      return None;
    }

    // Unsafe access to L1 for maximum performance
    // 为了最高性能，对 L1 进行不安全访问
    // Unsafe access to L1 for maximum performance
    // 为了最高性能，对 L1 进行不安全访问
    unsafe {
      let l1 = self.l1.get_ref();
      let group = l1.groups.get_unchecked(group_idx);
      let storage = &group.storage;

      if storage.is_empty() {
        return None;
      }

      // Fast Header access from fused storage.
      // 从融合存储中快速访问页眉。
      let header = Head::from_bytes(storage);

      // Fast path exit for empty groups.
      // 空组的快速退出路径。
      if header.is_empty() {
        return None;
      }

      let payload = storage.get_unchecked(Head::SIZE..);
      let pba = codec::decode(header, (lba % n as u64) as usize, payload, n);

      if pba == u64::MAX { None } else { Some(pba) }
    }
  }

  fn set(&mut self, lba: u64, pba: u64) {
    self.process_bg_results();

    // Tombstone Optimization:
    // Only write u64::MAX if there's actually something to delete.
    // 墓碑优化：只有在确实有内容可删除时才写入 u64::MAX。
    if pba == u64::MAX && self.get(lba).is_none() {
      return;
    }

    // COW: clone only if shared with background thread
    // COW: 仅当与后台线程共享时才克隆
    Arc::make_mut(&mut self.write_buffer).insert(lba, pba);
    if self.write_buffer.len() >= self.max_buffer_size {
      self.flush();
    }
  }

  #[cfg(feature = "stats")]
  fn mem(&mut self) -> usize {
    crate::mem::mem(self)
  }
}

impl<C: Conf> Ftl<C> {
  /// Inspects FTL to gather detailed statistics: segments, compression ratio, etc.
  /// 检查 FTL 以收集详细统计信息：segment、压缩率等。
  #[cfg(feature = "stats")]
  pub fn inspect_all_segments(&self) -> FtlStats {
    let l1 = unsafe { self.l1.get_ref() };
    crate::ftl::stats::FtlStats::collect::<C>(l1)
  }
}
