use std::time::Instant;

use jdb_ftl::{DefaultFtl, FtlTrait};
use jdb_ftl_example::{OP_WRITE, Result, TraceIter, open_bin};

fn main() -> Result<()> {
  let (file, name) = open_bin()?;
  println!("Tuning Trace: {}", name);

  let reader = std::io::BufReader::new(file);
  let mut trace = Vec::new();
  let mut max_lba = 0;

  for res in TraceIter::new(reader) {
    let rec = res?;
    if rec.lba > max_lba {
      max_lba = rec.lba;
    }
    if rec.op == OP_WRITE {
      trace.push((rec.lba, rec.pba));
    }
  }
  max_lba += 1;

  let mut ftl = DefaultFtl::new(max_lba);
  let start = Instant::now();
  for &(lba, pba) in &trace {
    ftl.set(lba, pba);
  }
  let duration = start.elapsed();

  let mem = ftl.mem();
  let ratio = (mem as f64 / (max_lba as f64 * 8.0)) * 100.0;

  println!("Time: {:.2?}", duration);
  println!("Memory: {} bytes", mem);
  println!("Compression Ratio: {:.2}%", ratio);

  Ok(())
}
