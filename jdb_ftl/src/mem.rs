#![cfg(feature = "stats")]

use rapidhash::RapidHashMap;

use crate::{Ftl, ftl::conf::Conf};

fn estimate_hashmap(m: &RapidHashMap<u64, u64>) -> usize {
  let cap = m.capacity();
  if cap == 0 {
    return 0;
  }
  let buckets = (cap as f64 / 0.875).ceil() as usize;
  let buckets = buckets.next_power_of_two();
  buckets * 17 + 64
}

pub(crate) fn mem<C: Conf>(ftl: &mut Ftl<C>) -> usize {
  ftl.sync();

  let l1 = unsafe { ftl.l1.get_ref() };

  // 1. L1 Groups Pointer Array
  let groups_array_size = std::mem::size_of_val(&*l1.groups);

  // 2. Combined storage (Header + Segs + Padding)
  // Each non-empty allocation has ~16 bytes malloc overhead.
  let payloads_data_size: usize = l1
    .groups
    .iter()
    .map(|g| {
      let len = g.storage.len();
      if len > 0 {
        len + 16 // Data + Malloc overhead
      } else {
        0
      }
    })
    .sum();

  // 4. HashMaps (Write Buffer + Flushing Buffers)
  // Each Arc/Allocation adds ~16 bytes overhead.
  let write_buffer_size = estimate_hashmap(&ftl.write_buffer) + 16;
  let flushing_size: usize = ftl
    .flushing
    .iter()
    .map(|buf| estimate_hashmap(buf) + 16)
    .sum();

  // 5. L1 Struct itself on heap + Arc metadata
  let l1_struct_on_heap = std::mem::size_of::<crate::ftl::l1::L1>() + 16;

  groups_array_size
    + payloads_data_size
    + write_buffer_size
    + flushing_size
    + l1_struct_on_heap
    + std::mem::size_of::<Ftl<C>>()
}
