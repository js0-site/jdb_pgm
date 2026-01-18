use std::time::Instant;
use jdb_ftl::{Frame, CompressionMode};
use rand::prelude::*;

fn main() {
  println!("--- Soft-PGM FTL Benchmark ---");

  // Setup: Create a Frame
  let mut frame = Frame::default();
  
  // Set headers to Linear mode (y = x)
  for (i, h) in frame.headers.iter_mut().enumerate() {
      h.base = (i * 32) as u64; // Group Base
      h.slope = 1;              // Linear slope
      h.config = (1 << 7);      // Mode::Linear
  }

  // Bench Read (L1 Hit Scenario)
  let n: usize = 1_000_000;
  let mut rng = StdRng::seed_from_u64(42);
  let indices: Vec<usize> = (0..n).map(|_| rng.random_range(0..512)).collect();

  let start = Instant::now();
  let mut chk = 0u64;
  for &idx in &indices {
      // Inline read simulation
      let val = frame.get(idx);
      chk ^= val;
  }
  let dur = start.elapsed();
  std::hint::black_box(chk);

  let ns_per_op = dur.as_nanos() as f64 / n as f64;
  let mops = n as f64 / 1_000_000.0 / dur.as_secs_f64();
  
  println!("Read Latency: {:.2} ns", ns_per_op);
  println!("Throughput:   {:.2} MOPS", mops);
  println!("Validation:   chk={}", chk);
}
