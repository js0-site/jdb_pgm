use std::{collections::VecDeque, sync::Arc, thread::JoinHandle};

use crossfire::spsc;
use rapidhash::RapidHashMap;

pub mod bench;
mod ftl_trait;
pub use ftl_trait::FtlTrait;
pub mod ftl;

pub mod error;
mod ftl_impl;
mod mem;

use ftl::{
  bg::{self, FlushResult, FlushTask},
  conf::Conf,
  frame::Head,
  l1::SharedL1,
};
pub use ftl_impl::DefaultFtl;

/// Value representing a deleted or unmapped LBA.
/// 表示已删除或未映射 LBA 的值。
pub const MAX: u64 = u64::MAX;

/// FTL Implementation using a Write Buffer and Background Flushing.
/// 使用写缓冲区和后台刷新的 FTL 实现。
pub struct Ftl<C: Conf> {
  pub(crate) write_buffer: Arc<RapidHashMap<u64, u64>>,
  pub(crate) max_buffer_size: usize,
  pub(crate) flushing: VecDeque<Arc<RapidHashMap<u64, u64>>>,
  pub(crate) l1: SharedL1,
  bg_tx: Option<crossfire::Tx<spsc::List<FlushTask>>>,
  // Main thread needs Async receive to yield properly in async contexts
  // 主线程需要异步接收以在异步上下文中正确让步
  bg_rx: crossfire::Rx<spsc::List<FlushResult>>,
  bg_thread: Option<JoinHandle<()>>,
  pub(crate) num_groups: usize,
  _marker: std::marker::PhantomData<C>,
}

unsafe impl<C: Conf> Send for Ftl<C> {}
unsafe impl<C: Conf> Sync for Ftl<C> {}

impl<C: Conf> Ftl<C> {
  pub fn new_with_capacity(capacity_lba: usize) -> Self {
    let l1 = SharedL1::new(capacity_lba, C::GROUP_SIZE);

    // Hybrid Channel Setup:
    // 混合通道设置：
    // 1. To Background: Blocking (BG thread sleeps on recv)
    // 1. 到后台：阻塞（后台线程在 recv 上休眠）
    let (tx_to_bg, rx_from_main) = spsc::unbounded_blocking();
    // 2. To Main: Async (Main thread awaits on recv)
    // 2. 到主线程：异步（主线程在 recv 上等待）
    let (tx_to_main, rx_from_bg) = spsc::unbounded_blocking();

    let l1_clone = l1.clone();
    let handle = std::thread::Builder::new()
      .name("ftl-bg".to_string())
      .spawn(move || {
        bg::run_bg::<C>(l1_clone, rx_from_main, tx_to_main);
      })
      .expect("Failed to spawn bg thread");
    let n = C::GROUP_SIZE;
    let num_groups = capacity_lba.div_ceil(n);
    Self {
      write_buffer: Arc::new(RapidHashMap::default()),
      max_buffer_size: C::WRITE_BUFFER_CAPACITY,
      flushing: VecDeque::new(),
      l1,
      bg_tx: Some(tx_to_bg),
      bg_rx: rx_from_bg,
      bg_thread: Some(handle),
      num_groups,
      _marker: std::marker::PhantomData,
    }
  }

  #[inline(always)]
  pub fn process_bg_results(&mut self) {
    while let Ok(res) = self.bg_rx.try_recv() {
      self.apply_result(res);
    }
  }

  pub fn flush(&mut self) {
    if self.write_buffer.is_empty() {
      return;
    }
    let old_buffer = std::mem::replace(&mut self.write_buffer, Arc::new(RapidHashMap::default()));

    // Only trigger flush if the queue is currently empty (meaning background is idle).
    // 仅当队列当前为空（意味着后台空闲）时才触发刷新。
    let should_trigger = self.flushing.is_empty();

    self.flushing.push_back(old_buffer.clone());

    if should_trigger && let Some(ref tx) = self.bg_tx {
      let _ = tx.send(FlushTask { buffer: old_buffer });
    }
  }

  #[inline]
  fn apply_result(&mut self, res: FlushResult) {
    match res {
      FlushResult::Done => {
        let _ = self.flushing.pop_front();
        // If there are pending buffers, process the next one.
        // 如果有挂起的缓冲区，请处理下一个。
        if let Some(next_buffer) = self.flushing.front()
          && let Some(ref tx) = self.bg_tx
        {
          let _ = tx.send(FlushTask {
            buffer: next_buffer.clone(),
          });
        }
      }
      res => {
        self.apply_single_result(res);
      }
    }
  }

  pub fn sync(&mut self) {
    if !self.write_buffer.is_empty() {
      self.flush();
    }
    while !self.flushing.is_empty() {
      // Blocking receive
      // 阻塞接收
      if let Ok(res) = self.bg_rx.recv() {
        self.apply_result(res);
      } else {
        break;
      }
    }
  }

  #[inline]
  fn apply_single_result(&mut self, res: FlushResult) {
    match res {
      FlushResult::Done => unreachable!(),
      FlushResult::Group {
        group_idx,
        header,
        chunks,
      } => {
        let l1_mut = unsafe { self.l1.get_mut() };

        // 1. Update Global Bitmap (Removed)
        // Global bitmap is no longer used.
        // 1. 更新全局位图（已删除）
        // 全局位图不再使用。

        // 2. Reconstruct Storage
        // 2. 重建存储
        let chunk_size: usize = chunks
          .iter()
          .map(|c| match c {
            bg::PayloadChunk::Reuse { len, .. } => *len as usize,
            bg::PayloadChunk::New(data) => data.len(),
          })
          .sum();

        if header.is_empty() {
          unsafe {
            let g = l1_mut.groups.get_unchecked_mut(group_idx);
            g.storage = Box::default();
          }
          return;
        }

        let mut new_storage = Vec::with_capacity(Head::SIZE + chunk_size + 16);
        new_storage.extend_from_slice(header.as_bytes());

        for chunk in chunks {
          match chunk {
            bg::PayloadChunk::Reuse { offset, len } => {
              let old_s = unsafe { &l1_mut.groups.get_unchecked(group_idx).storage };
              let old_p_offset = Head::SIZE + offset as usize;
              let len = len as usize;

              let src_slice = if old_p_offset < old_s.len() {
                let end = (old_p_offset + len).min(old_s.len());
                &old_s[old_p_offset..end]
              } else {
                &[]
              };
              new_storage.extend_from_slice(src_slice);

              // If the requested length was greater than available in old_s,
              // extend with zeros for the remaining part.
              // 如果请求的长度大于 old_s 中可用的长度，
              // 则用零填充剩余部分。
              let copied_len = src_slice.len();
              if copied_len < len {
                new_storage.extend(std::iter::repeat_n(0, len - copied_len));
              }
            }
            bg::PayloadChunk::New(data) => {
              new_storage.extend_from_slice(&data);
            }
          }
        }

        // Add 16 bytes of padding for safe unaligned reads.
        // 添加 16 字节的填充以实现安全的未对齐读取。
        new_storage.extend_from_slice(&[0u8; 16]);

        unsafe {
          let g = l1_mut.groups.get_unchecked_mut(group_idx);
          g.storage = new_storage.into_boxed_slice();
        }
      }
    }
  }
}

impl<C: Conf> Drop for Ftl<C> {
  fn drop(&mut self) {
    self.bg_tx.take();
    if let Some(handle) = self.bg_thread.take() {
      let _ = handle.join();
    }
  }
}
