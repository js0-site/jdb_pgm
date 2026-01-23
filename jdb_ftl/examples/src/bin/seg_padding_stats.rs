use std::{
  io::{BufReader, Seek, SeekFrom},
  time::Instant,
};

use hdrhistogram::Histogram;
use jdb_ftl::{DefaultFtl, FtlTrait};
use jdb_ftl_example::{OP_WRITE, Result, TraceIter, open_bin};

fn main() -> Result<()> {
  let (file, name) = open_bin()?;
  println!("Analyzing Trace: {}", name);
  let mut reader = BufReader::new(file);

  // 1. Scan for max LBA
  // 1. 扫描获取最大 LBA
  let mut max_lba = 0u64;
  for res in TraceIter::new(&mut reader) {
    let rec = res?;
    if rec.lba > max_lba {
      max_lba = rec.lba;
    }
  }
  max_lba += 1;

  // 2. Initialize and Replay
  // 2. 初始化与重放
  let mut ftl = DefaultFtl::new(max_lba);
  reader.get_mut().seek(SeekFrom::Start(0))?;

  let start = Instant::now();
  for res in TraceIter::new(reader) {
    let rec = res?;
    if rec.op == OP_WRITE {
      ftl.set(rec.lba, rec.pba);
    }
  }
  println!("Replay took: {:.2?}", start.elapsed());

  // 3. Inspect and Stats
  // 3. 检查与统计
  ftl.mem();
  let stats = ftl.inspect_all_segments();
  let segments = &stats.segment_lengths;

  if segments.is_empty() {
    println!("No segments found.");
    return Ok(());
  }

  let mut hist = Histogram::<u64>::new_with_bounds(1, 1_000_000, 3).unwrap();
  for &len in segments {
    hist.record(len as u64).unwrap();
  }

  println!("\n--- Segment Length Distribution ---");
  println!(
    "Avg: {:.2}",
    segments.iter().sum::<usize>() as f64 / segments.len() as f64
  );
  println!("P50: {}", hist.value_at_quantile(0.5));
  println!("P90: {}", hist.value_at_quantile(0.9));
  println!("P99: {}", hist.value_at_quantile(0.99));
  println!("Max: {}", hist.max());

  Ok(())
}
