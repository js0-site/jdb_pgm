use crate::base::Bench;

pub struct ArrayBench(Box<[u64]>);

impl Bench for ArrayBench {
  const NAME: &'static str = "Array";

  fn build(data: &[u64]) -> Self {
    Self(data.to_vec().into_boxed_slice())
  }

  fn size_in_bytes(&self) -> usize {
    self.0.len() * 8
  }

  fn get(&self, index: usize) -> u64 {
    unsafe { *self.0.get_unchecked(index) }
  }

  fn iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_ {
    self.0[range].iter().copied()
  }

  fn rev_iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_ {
    self.0[range].iter().rev().copied()
  }
}
