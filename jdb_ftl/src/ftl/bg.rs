use std::sync::Arc;

use rapidhash::RapidHashMap;

use crate::ftl::{codec::encode, conf::Conf, frame::Head, l1::SharedL1};

/// Type alias for shard index
/// 分片索引类型别名
pub type ShardIdx = usize;

/// Type alias for fragmentation score (delete count)
/// 碎片化分数类型别名（删除计数）
pub type Score = usize;

/// Task data for the background flush thread.
/// 后台刷新线程的任务数据。
pub struct FlushTask {
  pub buffer: Arc<RapidHashMap<u64, u64>>,
}

/// A chunk of a group's payload, either reused from the previous version or newly encoded.
/// 组有效负载的块，可以是上一版本的重用，也可以是新编码的。
pub enum PayloadChunk {
  Reuse { offset: u32, len: u32 },
  New(Vec<u8>),
}

/// Result of a single group flush operation.
/// 单个组刷新操作的结果。
pub enum FlushResult {
  Group {
    group_idx: usize,
    header: Head,
    chunks: Vec<PayloadChunk>,
  },
  Done,
}

pub fn run_bg<C: Conf>(
  l1: SharedL1,
  rx: crossfire::Rx<crossfire::spsc::List<FlushTask>>,
  tx: crossfire::Tx<crossfire::spsc::List<FlushResult>>,
) {
  let n = C::GROUP_SIZE;
  // Hoist allocations to reuse memory across tasks and groups
  // 提升分配以跨任务和组复用内存
  let mut group_ppas = vec![u64::MAX; n];
  let mut lba_offsets = Vec::with_capacity(n);
  let mut dense_ppas = Vec::with_capacity(n);
  let mut dense_dirty_map = Vec::with_capacity(n);
  let mut lba_ef = Vec::with_capacity(256); // Estimated size

  while let Ok(task) = rx.recv() {
    let mut updates: Vec<(u64, u64)> = task.buffer.iter().map(|(&k, &v)| (k, v)).collect();
    updates.sort_unstable_by_key(|&(lba, _)| lba);

    if updates.is_empty() {
      let _ = tx.send(FlushResult::Done);
      continue;
    }

    let mut i = 0;
    while i < updates.len() {
      let first_lba = unsafe { updates.get_unchecked(i).0 };
      let group_idx = (first_lba / n as u64) as usize;
      let group_start_lba = (group_idx * n) as u64;
      let group_end_lba = group_start_lba + n as u64;

      let start_update_idx = i;
      while i < updates.len() {
        let u_lba = unsafe { updates.get_unchecked(i).0 };
        if u_lba >= group_end_lba {
          break;
        }
        i += 1;
      }
      let group_updates = unsafe { updates.get_unchecked(start_update_idx..i) };

      let l1_ref = unsafe { l1.get_ref() };
      if group_idx >= l1_ref.groups.len() {
        continue;
      }

      // Reuse vectors
      // 复用向量
      lba_offsets.clear();
      dense_ppas.clear();
      dense_dirty_map.clear();
      lba_ef.clear();

      let res = process_group::<C>(
        l1_ref,
        group_idx,
        group_start_lba,
        group_updates,
        &mut group_ppas,
        &mut lba_offsets,
        &mut dense_ppas,
        &mut dense_dirty_map,
        &mut lba_ef,
      );

      if let Some(r) = res
        && tx.send(r).is_err()
      {
        return;
      }
    }

    let _ = tx.send(FlushResult::Done);
  }
}

#[allow(clippy::too_many_arguments)]
pub fn process_group<C: Conf>(
  l1: &crate::ftl::l1::L1,
  group_idx: usize,
  group_start_lba: u64,
  group_updates: &[(u64, u64)],
  group_ppas: &mut [u64],
  lba_offsets: &mut Vec<u16>,
  dense_ppas: &mut Vec<u64>,
  dense_dirty_map: &mut Vec<bool>,
  lba_ef: &mut Vec<u8>,
) -> Option<FlushResult> {
  let group = unsafe { l1.groups.get_unchecked(group_idx) };
  let storage = &group.storage;

  if storage.is_empty() {
    group_ppas.fill(u64::MAX);
  } else {
    // Safety: storage is not empty, Head::SIZE is 2.
    // 安全性：storage 不为空，Head::SIZE 为 2。
    let header = unsafe { Head::from_bytes(storage) };
    let payload = unsafe { storage.get_unchecked(Head::SIZE..) };

    if header.is_empty() {
      group_ppas.fill(u64::MAX);
    } else {
      // Just decode directly into the full array
      // 直接解码到完整数组中
      crate::ftl::codec::decode_group(header, payload, group_ppas);
    }
  }

  // 3. Merge New Updates
  // 3. 合并新更新
  let mut dirty_map = vec![false; C::GROUP_SIZE];
  for &(lba, pba) in group_updates {
    let local_idx = (lba - group_start_lba) as usize;
    unsafe {
      *group_ppas.get_unchecked_mut(local_idx) = pba;
    }
    if local_idx < dirty_map.len() {
      dirty_map[local_idx] = true;
    }
  }

  // 4. Re-encode
  // Extract only valid (LBA_offset, PBA) pairs in a single pass.
  // 在一次遍历中仅提取有效的 (LBA_offset, PBA) 对。
  // dense_dirty_map passed from caller, already cleared
  for (off, &ppa) in group_ppas.iter().enumerate() {
    if ppa != u64::MAX {
      lba_offsets.push(off as u16);
      dense_ppas.push(ppa);
      dense_dirty_map.push(dirty_map[off]);
    }
  }

  if lba_offsets.is_empty() {
    return Some(FlushResult::Group {
      group_idx,
      header: Head::new(), // Empty header
      chunks: Vec::new(),
    });
  }

  // Encode LBA offsets using Elias-Fano (Sparse Index)
  // 使用 Elias-Fano（稀疏索引）编码 LBA 偏移量
  let lba_offsets_len_bytes = (lba_offsets.len() as u16).to_le_bytes();
  let encoded_lba_offsets = crate::ftl::codec::ef::encode(lba_offsets, C::GROUP_SIZE);

  lba_ef.extend_from_slice(&lba_offsets_len_bytes);
  lba_ef.extend_from_slice(&encoded_lba_offsets);

  // Extract old PGM payload for reuse if available
  let mut old_pgm_payload = None;
  if !storage.is_empty() {
    let header = unsafe { Head::from_bytes(storage) };
    if !header.is_empty() {
      let payload = unsafe { storage.get_unchecked(Head::SIZE..) };
      // Read LBA EF count to calculate offset
      let n_valid = unsafe { u16::from_le_bytes(*(payload.as_ptr() as *const [u8; 2])) } as usize;
      let ef_bytes = 2 + crate::ftl::codec::ef::byte_len(n_valid, C::GROUP_SIZE);
      if payload.len() > ef_bytes {
        old_pgm_payload = Some(&payload[ef_bytes..]);
      }
    }
  }

  let (new_header, mut chunks) = encode(
    dense_ppas, // Use dense array
    // 使用密集数组
    &dense_dirty_map,
    old_pgm_payload,
    C::PGM_EPSILON,
  );

  // If not empty, prepend LBA EF index
  // 如果不为空，则预置 LBA EF 索引
  if !new_header.is_empty() {
    chunks.insert(0, PayloadChunk::New(lba_ef.clone()));
  }

  Some(FlushResult::Group {
    group_idx,
    header: new_header,
    chunks,
  })
}
