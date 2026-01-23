use std::hint::black_box;

use criterion::{BenchmarkId, measurement::WallTime};
use rand::{Rng, SeedableRng, rngs::StdRng};

pub const SEED: u64 = 42;

/// Trait for benchmark implementations
/// 基准测试实现的特征
pub trait Benchmarkable: Sized {
  const NAME: &'static str;

  fn build(data: &[u64], epsilon: Option<usize>) -> Self;
  fn query(&self, data: &[u64], key: u64) -> Option<usize>;

  fn bench_name(epsilon: Option<usize>) -> String {
    if let Some(eps) = epsilon {
      format!("{}_{}", Self::NAME, eps)
    } else {
      Self::NAME.to_string()
    }
  }
}

/// Generate sorted data with random gaps (consistent with accuracy.rs)
/// 生成带有随机间隔的已排序数据（与 accuracy.rs 保持一致）
#[inline]
pub fn gen_seq(size: usize) -> Vec<u64> {
  let mut rng = StdRng::seed_from_u64(SEED);
  let mut cur = 0u64;
  (0..size)
    .map(|_| {
      cur += rng.random_range(1..100);
      cur
    })
    .collect()
}

/// Generate random queries from actual data (ensures all queries hit)
/// 从实际数据中生成随机查询（确保所有查询都命中）
#[inline]
pub fn gen_queries(data: &[u64], count: usize) -> Vec<u64> {
  let mut rng = StdRng::seed_from_u64(SEED);
  (0..count)
    .map(|_| data[rng.random_range(0..data.len())])
    .collect()
}

/// Benchmark query time for a given implementation
/// 对给定实现的查询时间进行基准测试
pub fn bench_query_impl<T: Benchmarkable>(
  group: &mut criterion::BenchmarkGroup<WallTime>,
  data: &[u64],
  queries: &[u64],
  input_value: usize,
  eps: Option<usize>,
) {
  let idx = T::build(data, eps);
  group.bench_with_input(
    BenchmarkId::new(T::bench_name(eps), input_value),
    &(data, queries),
    |b, (data, queries)| {
      b.iter(|| {
        for &q in queries.iter() {
          black_box(idx.query(data, q));
        }
      })
    },
  );
}

/// Benchmark construction time for a given implementation
/// 对给定实现的构建时间进行基准测试
pub fn bench_build_impl<T: Benchmarkable>(
  group: &mut criterion::BenchmarkGroup<WallTime>,
  data: &[u64],
  size: usize,
  eps: Option<usize>,
) {
  group.bench_with_input(
    BenchmarkId::new(T::bench_name(eps), size),
    &data,
    |b, data| b.iter(|| black_box(T::build(data, eps))),
  );
}
