use std::{
  fs::File,
  io::Read,
  path::PathBuf,
  time::{Duration, Instant},
};

use hdrhistogram::Histogram;
use jdb_ftl::FtlTrait;
#[cfg(feature = "bench_base")]
use jdb_ftl::bench::base::Base;
use mimalloc::MiMalloc;
use sonic_rs::JsonValueTrait;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod name;
use name::Name;

const OP_READ: u8 = 0;
const OP_WRITE: u8 = 1;

struct OpRecord {
  op: u8,
  lba: u64,
  pba: u64,
}

fn load_trace() -> (Vec<OpRecord>, PathBuf) {
  let bin = std::env::var("BIN").unwrap_or_else(|_| "quick".to_string());
  let path = PathBuf::from(format!("data/{}.bin", bin));

  if !path.exists() {
    panic!(
      "Trace file {:?} not found! Please run 'bun init.js' first.",
      path
    );
  }

  let mut file = File::open(&path).expect("Failed to open trace file");
  let mut buf = Vec::new();
  file
    .read_to_end(&mut buf)
    .expect("Failed to read trace file");

  let data = buf
    .chunks_exact(16)
    .map(|c| {
      let lba = u64::from_le_bytes(c[0..8].try_into().unwrap());
      let meta = u64::from_le_bytes(c[8..16].try_into().unwrap());
      OpRecord {
        op: (meta >> 60) as u8,
        lba,
        pba: meta & 0x0FFFFFFFFFFFFFFF,
      }
    })
    .collect();

  (data, path)
}

fn replay<T: FtlTrait + Name>(name: &str, trace: &[OpRecord], cap: u64) {
  let mut ftl = T::new(cap);

  let mut read_hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();
  let mut write_hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();

  let mut read_count = 0u64;
  let mut write_count = 0u64;

  let mut read_time = Duration::ZERO;
  let mut write_time = Duration::ZERO;

  println!("Replaying trace for {}...", name);
  let total_start = Instant::now();

  for rec in trace {
    match rec.op {
      OP_READ => {
        let start = Instant::now();
        let _ = ftl.get(rec.lba);
        let elapsed = start.elapsed();
        read_time += elapsed;
        read_count += 1;
        let ns = elapsed.as_nanos() as u64;
        read_hist.record(ns.max(1)).unwrap();
      }
      OP_WRITE => {
        let start = Instant::now();
        ftl.set(rec.lba, rec.pba); // Use correct PBA
        let elapsed = start.elapsed();
        write_time += elapsed;
        write_count += 1;
        let ns = elapsed.as_nanos() as u64;
        write_hist.record(ns.max(1)).unwrap();
      }
      _ => {}
    }
  }

  let total_duration = total_start.elapsed();
  let total_ops = read_count + write_count;

  // JSON output for parser
  println!(
    "{{\"type\": \"replay_summary\", \"name\": \"{}\", \"total_time_ms\": {}, \"total_ops\": {}}}",
    name,
    total_duration.as_millis(),
    total_ops
  );

  let print_op = |op_name: &str, count: u64, total_time: Duration, hist: &Histogram<u64>| {
    if count == 0 {
      return;
    }
    let avg_ns = total_time.as_nanos() as f64 / count as f64;
    let p99_ns = hist.value_at_quantile(0.99);
    let count_pct = (count as f64 / total_ops as f64) * 100.0;
    let time_pct = (total_time.as_nanos() as f64 / total_duration.as_nanos() as f64) * 100.0;
    let throughput = count as f64 / total_time.as_secs_f64();

    println!(
      "{{\"type\": \"op_stat\", \"name\": \"{}\", \"op\": \"{}\", \"count\": {}, \"count_pct\": {:.2}, \"time_pct\": {:.2}, \"avg_ns\": {:.2}, \"p99_ns\": {}, \"ops_per_sec\": {:.2}}}",
      name, op_name, count, count_pct, time_pct, avg_ns, p99_ns, throughput
    );
  };

  print_op("get", read_count, read_time, &read_hist);
  print_op("set", write_count, write_time, &write_hist);

  #[cfg(feature = "stats")]
  {
    let mem_bytes = ftl.mem();
    let mem_mb = mem_bytes as f64 / 1024.0 / 1024.0;
    println!(
      "{{\"type\": \"memory_usage\", \"name\": \"{}\", \"mem_mb\": {:.2}, \"mem_bytes\": {}}}",
      name, mem_mb, mem_bytes
    );
  }
}

fn main() {
  let (trace, path) = load_trace();
  let bin_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("quick");

  let config_path = PathBuf::from(format!("data/{}.json", bin_name));
  let cap = if config_path.exists() {
    let content = std::fs::read_to_string(&config_path).expect("Failed to read config");
    // 手动解析 JSON，从 sonic_rs::Value 中提取 max_lba
    let config: sonic_rs::Value = sonic_rs::from_str(&content).expect("Failed to parse config");
    let max_lba_str = config["max_lba"]
      .as_str()
      .expect("Failed to get max_lba as string");
    max_lba_str.parse::<u64>().expect("Failed to parse max_lba") + 1
  } else {
    let max_lba = trace.iter().map(|r| r.lba).max().unwrap_or(0);
    max_lba + 1
  };

  // Replay for Base
  #[cfg(feature = "bench_base")]
  replay::<Base>(Base::NAME, &trace, cap);

  // Replay for Ftl
  #[cfg(feature = "bench_ftl")]
  use jdb_ftl::DefaultFtl;
  #[cfg(feature = "bench_ftl")]
  replay::<DefaultFtl>(DefaultFtl::NAME, &trace, cap);
}
