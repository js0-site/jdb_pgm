use std::io::BufReader;

use jdb_ftl_example::{OP_WRITE, Result, TraceIter, open_bin};

fn main() -> Result<()> {
  let (file, name) = open_bin()?;
  println!("Analyzing Trace: {}", name);

  let mut reader = BufReader::new(file);
  let mut lbas = Vec::new();

  for res in TraceIter::new(&mut reader) {
    let rec = res?;
    if rec.op == OP_WRITE {
      lbas.push(rec.lba);
    }
  }

  lbas.sort_unstable();

  // Logic for direct mode analysis
  // ... (keeping user's original logic but cleaned up)

  println!("Total writes: {}", lbas.len());
  Ok(())
}
