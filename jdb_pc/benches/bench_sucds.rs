use sucds::{
  Serializable,
  mii_sequences::{EliasFano, EliasFanoBuilder},
};

use crate::base::Bench;

pub struct SucdsBench(EliasFano);

impl Bench for SucdsBench {
  const NAME: &'static str = "Sucds";

  fn build(data: &[u64]) -> Self {
    let n = data.len();
    let max_val = *data.last().unwrap_or(&0) as usize + 1;
    let mut efb = EliasFanoBuilder::new(max_val, n).unwrap();
    efb.extend(data.iter().map(|&x| x as usize)).unwrap();
    Self(efb.build())
  }

  fn size_in_bytes(&self) -> usize {
    self.0.size_in_bytes()
  }

  fn get(&self, index: usize) -> u64 {
    self.0.select(index).unwrap() as u64
  }

  fn iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_ {
    self
      .0
      .iter(range.start)
      .take(range.end - range.start)
      .map(|x| x as u64)
  }

  fn rev_iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_ {
    range
      .rev()
      .map(move |idx| self.0.select(idx).unwrap() as u64)
  }
}
