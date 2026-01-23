use std::{fs::File, io::Read, path::PathBuf};

use jdb_ftl::{DefaultFtl, FtlTrait};
use rapidhash::{HashMapExt, RapidHashMap};

struct OpRecord {
  op: u8,
  lba: u64,
  pba: u64,
}

#[test]
fn test_quick_trace_correctness() {
  let bin = std::env::var("BIN").unwrap_or_else(|_| "quick".to_string());

  let trace_path = PathBuf::from(format!("data/{}.bin", bin));

  println!("{}", trace_path.display());
  if !trace_path.exists() {
    println!("Skipping test: {:?} not found", trace_path);
    return;
  }

  // 2. Load Trace
  let mut file = File::open(&trace_path).expect("Failed to open trace file");
  let mut buf = Vec::new();
  file
    .read_to_end(&mut buf)
    .expect("Failed to read trace file");

  let trace: Vec<OpRecord> = buf
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

  // Calculate Max LBA from trace
  let max_lba_in_trace = trace
    .iter()
    .map(|OpRecord { lba, .. }| *lba)
    .max()
    .unwrap_or(0);
  // Be generous
  let cap = max_lba_in_trace + 1024 * 1024;

  println!(
    "Trace loaded. Max LBA: {}, Cap set to: {}",
    max_lba_in_trace, cap
  );

  // 3. Replay and Verify
  let mut ftl = DefaultFtl::new(cap);
  let mut shadow = RapidHashMap::new();

  println!("Replaying ... operations...");
  for rec in &trace {
    if rec.op == 1 {
      // WRITE
      // Use the PBA from the trace (Golden Truth)
      ftl.set(rec.lba, rec.pba);
      shadow.insert(rec.lba, rec.pba);
    }
  }

  // 4. Force Flush
  println!("Flushing L0 to L1...");
  ftl.flush();

  // 5. Final Verification
  println!("Verifying {} unique LBAs...", shadow.len());
  let mut sorted_lbas: Vec<u64> = shadow.keys().cloned().collect();
  sorted_lbas.sort();

  for lba in sorted_lbas {
    let expected_pba = shadow[&lba];
    let actual = ftl.get(lba);
    if actual != Some(expected_pba) {
      eprintln!(
        "FAIL at LBA {}: expected PBA={} (0x{:016x}), actual {:?}",
        lba, expected_pba, expected_pba, actual
      );
      // Panic immediately on first failure to see the error.
      panic!("Verification failed at LBA {}", lba);
    }
  }
  println!("Verification successful!");
}
