pub trait Bench: Sized {
  const NAME: &'static str;
  fn build(data: &[u64]) -> Self;
  fn size_in_bytes(&self) -> usize;
  fn get(&self, index: usize) -> u64;
  fn iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_;
  fn rev_iter_range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = u64> + '_;
}
