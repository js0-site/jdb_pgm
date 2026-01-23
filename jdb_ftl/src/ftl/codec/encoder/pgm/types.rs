use crate::ftl::{codec::optimal::FitResult, seg::Seg};

pub struct OldSegInfo<'a> {
  pub seg: Seg,
  pub start: u16,
  pub end: u16,
  pub byte_offset: usize,
  pub len_bytes: usize,
  pub _phantom: std::marker::PhantomData<&'a ()>,
}

pub enum Plan {
  Reuse {
    old_idx: usize,
  },
  New {
    fit: FitResult,
    len: u16,
    max_res: u64,
  },
}
