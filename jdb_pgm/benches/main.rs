//! Criterion benchmark comparing Pgm-Index vs binary search vs pgm_index crate
//! Criterion 基准测试：Pgm-Index vs 二分查找 vs pgm_index crate

#[cfg(feature = "bench")]
#[path = "parts/bench_binary.rs"]
mod bench_binary;
#[cfg(feature = "bench")]
#[path = "parts/bench_btreemap.rs"]
mod bench_btreemap;
#[path = "parts/bench_common.rs"]
mod bench_common;
#[cfg(feature = "bench")]
#[path = "parts/bench_external_pgm.rs"]
mod bench_external_pgm;
#[path = "parts/bench_jdb_pgm.rs"]
mod bench_jdb_pgm;

use std::time::Duration;

#[cfg(feature = "bench")]
use bench_binary::BinarySearch;
#[cfg(feature = "bench")]
use bench_btreemap::BTreeMapIndex;
use bench_common::{bench_build_impl, bench_query_impl, gen_queries, gen_seq};
#[cfg(feature = "bench")]
use bench_external_pgm::ExternalPgm;
use bench_jdb_pgm::JdbPgm;
use criterion::{Criterion, Throughput, criterion_group, criterion_main, measurement::WallTime};

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const SAMPLE_SIZE: usize = 20;
const DATA_SIZES: &[usize] = &[1_000_000];
const EPSILONS: &[usize] = &[32, 64, 128];

fn setup_group<'a>(c: &'a mut Criterion, name: &str) -> criterion::BenchmarkGroup<'a, WallTime> {
  let mut group = c.benchmark_group(name);
  group
    .sample_size(SAMPLE_SIZE)
    .warm_up_time(Duration::from_millis(100))
    .measurement_time(Duration::from_secs(1));
  group
}

fn bench_single(c: &mut Criterion) {
  let mut group = setup_group(c, "single_lookups");

  for &size in DATA_SIZES {
    let data = gen_seq(size);
    let queries = gen_queries(&data, 1000);
    group.throughput(Throughput::Elements(queries.len() as u64));

    #[cfg(feature = "bench")]
    {
      bench_query_impl::<BinarySearch>(&mut group, &data, &queries, size, None);
      bench_query_impl::<BTreeMapIndex>(&mut group, &data, &queries, size, None);
    }

    for &eps in EPSILONS {
      bench_query_impl::<JdbPgm>(&mut group, &data, &queries, size, Some(eps));
      #[cfg(feature = "bench")]
      bench_query_impl::<ExternalPgm>(&mut group, &data, &queries, size, Some(eps));
    }
  }
  group.finish();
}

#[cfg(feature = "bench")]
fn bench_batch(c: &mut Criterion) {
  let mut group = setup_group(c, "batch_lookups");

  let data = gen_seq(1_000_000);

  for batch in [100, 1_000, 10_000] {
    let queries = gen_queries(&data, batch);
    group.throughput(Throughput::Elements(batch as u64));

    bench_query_impl::<BinarySearch>(&mut group, &data, &queries, batch, None);
    bench_query_impl::<JdbPgm>(&mut group, &data, &queries, batch, Some(64));
  }
  group.finish();
}

fn bench_build(c: &mut Criterion) {
  let mut group = setup_group(c, "build_time");

  for &size in DATA_SIZES {
    let data = gen_seq(size);
    group.throughput(Throughput::Elements(size as u64));

    for &eps in EPSILONS {
      bench_build_impl::<JdbPgm>(&mut group, &data, size, Some(eps));

      #[cfg(feature = "bench")]
      bench_build_impl::<ExternalPgm>(&mut group, &data, size, Some(eps));
    }
  }
  group.finish();
}

#[cfg(feature = "bench")]
fn bench_compare(c: &mut Criterion) {
  let mut group = setup_group(c, "jdb_vs_external");

  for &size in DATA_SIZES {
    let data = gen_seq(size);
    let queries = gen_queries(&data, 1000);
    group.throughput(Throughput::Elements(queries.len() as u64));

    for &eps in EPSILONS {
      bench_query_impl::<JdbPgm>(&mut group, &data, &queries, size, Some(eps));
      bench_query_impl::<ExternalPgm>(&mut group, &data, &queries, size, Some(eps));
    }
  }
  group.finish();
}

#[cfg(feature = "bench")]
criterion_group!(
  benches,
  bench_single,
  bench_batch,
  bench_build,
  bench_compare
);

#[cfg(not(feature = "bench"))]
criterion_group!(benches, bench_single, bench_build);

criterion_main!(benches);
