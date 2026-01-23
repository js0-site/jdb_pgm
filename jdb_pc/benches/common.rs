use criterion::{BenchmarkGroup, measurement::WallTime};
use rand::prelude::*;
use rand_distr::Distribution;

use crate::base::Bench;

pub const N_QUERIES: usize = 5_000_000;
pub const SEED: u64 = 12345;

pub struct Data {
  pub name: &'static str,
  pub data: Vec<u64>,
}

pub fn gen_key_offsets(n: usize) -> Data {
  let mut rng = StdRng::seed_from_u64(42);
  let zipf = rand_distr::Zipf::new(100.0, 1.5).unwrap();
  let mut cur = 0u64;
  let data: Vec<u64> = (0..n)
    .map(|_| {
      cur += zipf.sample(&mut rng) as u64 + 16;
      cur
    })
    .collect();

  Data {
    name: "KeyOffsets",
    data,
  }
}

pub fn gen_doc_ids(n: usize) -> Data {
  let mut rng = StdRng::seed_from_u64(42);
  let mut cur = 0u64;
  let data = (0..n)
    .map(|_| {
      cur += rng.random_range(1..100);
      cur
    })
    .collect();

  Data {
    name: "DocIds",
    data,
  }
}

pub fn run_bench<T: Bench>(group: &mut BenchmarkGroup<WallTime>, dataset_name: &str, data: &[u64]) {
  let n = data.len();
  println!("DEBUG: Dataset Size N = {}", n);

  // Build
  group.bench_function(format!("{}/Build", T::NAME), |b| b.iter(|| T::build(data)));

  let index = T::build(data);
  let size = index.size_in_bytes();

  // Custom output for size
  println!(
    r#"{{"reason": "custom-metric", "id": "{}/{}/Size", "estimate": {}}}"#,
    dataset_name,
    T::NAME,
    size
  );

  // Random Get
  let mut rng = StdRng::seed_from_u64(SEED);
  let indices: Vec<usize> = (0..N_QUERIES.min(n))
    .map(|_| rng.random_range(0..n))
    .collect();
  group.bench_function(format!("{}/RandomGet", T::NAME), |b| {
    b.iter(|| {
      let mut last = 0u64;
      for &idx in &indices {
        // Force dependency: target depends on last (which comes from previous get).
        // Using `last & 1` ensures the CPU cannot optimize away the dependency (unlike `& 0`).
        // Since n is even (131,072,000), idx ^ 1 is always within bounds [0, n).
        let target = idx ^ (last as usize & 1);
        last = index.get(target);
      }
      std::hint::black_box(last)
    })
  });

  // Iter
  // Iter
  group.bench_function(format!("{}/Iter", T::NAME), |b| {
    b.iter(|| {
      let mut chk = 0;
      for val in index.iter_range(0..n) {
        chk ^= val;
      }
      std::hint::black_box(chk)
    })
  });

  // RevIter (always supported now)
  group.bench_function(format!("{}/RevIter", T::NAME), |b| {
    b.iter(|| {
      let mut chk = 0;
      for val in index.rev_iter_range(0..n) {
        chk ^= val;
      }
      std::hint::black_box(chk)
    })
  });
}
