use criterion::{Criterion, criterion_group, criterion_main};

mod base;
mod common;

#[cfg(feature = "bench-array")]
mod bench_array;
#[cfg(feature = "bench-pc")]
mod bench_pc;
#[cfg(feature = "bench-sucds")]
mod bench_sucds;

fn benchmarks(c: &mut Criterion) {
  // 1000 MiB elements
  let n = 131_072_000;

  let scenarios = [common::gen_key_offsets(n), common::gen_doc_ids(n)];

  for dataset in &scenarios {
    let mut group = c.benchmark_group(dataset.name);
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(100));
    group.measurement_time(std::time::Duration::from_secs(10));

    #[cfg(feature = "bench-pc")]
    common::run_bench::<bench_pc::PcBench>(&mut group, dataset.name, &dataset.data);

    #[cfg(feature = "bench-sucds")]
    common::run_bench::<bench_sucds::SucdsBench>(&mut group, dataset.name, &dataset.data);

    #[cfg(feature = "bench-array")]
    common::run_bench::<bench_array::ArrayBench>(&mut group, dataset.name, &dataset.data);

    group.finish();
  }
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
