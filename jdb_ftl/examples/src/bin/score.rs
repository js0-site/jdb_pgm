use std::time::Instant;

use hdrhistogram::Histogram;
use jdb_ftl::{DefaultFtl, FtlTrait};
use jdb_ftl_example::{OP_READ, Result, TraceIter, open_bin};

fn main() -> Result<()> {
  let (file, name) = open_bin()?;
  println!("Scoring Trace: {}", name);

  let mut reader = std::io::BufReader::new(file);
  let mut max_lba = 0u64;
  let mut trace = Vec::new();

  // Load all into memory for scoring (speed matters here)
  // 全部加载到内存中进行评分（速度很重要）
  for res in TraceIter::new(&mut reader) {
    let rec = res?;
    if rec.lba > max_lba {
      max_lba = rec.lba;
    }
    trace.push(rec);
  }
  max_lba += 1;

  let mut ftl = DefaultFtl::new(max_lba);
  let mut read_hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();

  let start = Instant::now();
  for rec in &trace {
    if rec.op == OP_READ {
      let t = Instant::now();
      let _ = ftl.get(rec.lba);
      read_hist.record(t.elapsed().as_nanos() as u64).unwrap();
    } else {
      ftl.set(rec.lba, rec.pba);
    }
  }

  let duration = start.elapsed();
  let throughput = (trace.len() as f64 * 8.0) / (1024.0 * 1024.0) / duration.as_secs_f64();
  let p99 = read_hist.value_at_quantile(0.99) as f64;

  println!("Throughput: {:.2} MB/s", throughput);
  println!("P99 Latency: {:.2} ns", p99);
  println!(
    "Memory Ratio: {:.2}%",
    (ftl.mem() as f64 / (max_lba as f64 * 8.0)) * 100.0
  );

  Ok(())
}
