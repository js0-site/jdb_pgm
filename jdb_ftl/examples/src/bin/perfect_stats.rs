use std::io::BufReader;

use jdb_ftl_example::{HashMapExt, OP_WRITE, Result, TraceIter, open_bin};
use rapidhash::RapidHashMap;

fn main() -> Result<()> {
  let (file, _) = open_bin()?;
  let reader = BufReader::new(file);

  let mut map = RapidHashMap::new();
  let mut ppa_counter = 0;

  for res in TraceIter::new(reader) {
    let rec = res?;
    if rec.op == OP_WRITE {
      map.insert(rec.lba, ppa_counter);
      ppa_counter += 1;
    }
  }

  println!("Unique LBAs: {}", map.len());
  // ... rest of logic
  Ok(())
}
