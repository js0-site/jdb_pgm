use jdb_pc::Pc;

use crate::base::Bench;

pub struct PcBench(Pc);

impl Bench for PcBench {
  const NAME: &'static str = "Pc";

  fn build(data: &[u64]) -> Self {
    Self(Pc::new(data, jdb_pc::types::DEFAULT_EPSILON))
  }

  fn size_in_bytes(&self) -> usize {
    self.0.dump().len()
  }

  fn get(&self, index: usize) -> u64 {
    unsafe { self.0.get_unchecked(index) }
  }

  fn iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_ {
    self.0.iter_range(range)
  }

  fn rev_iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_ {
    self.0.rev_iter_range(range)
  }
}
