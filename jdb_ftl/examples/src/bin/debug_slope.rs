use jdb_ftl::{DefaultFtl, FtlTrait};

fn main() {
  let mut ftl = DefaultFtl::new(10000);

  // Simulate some writes that might trigger the bug
  // LBA 1917 is in the first few groups
  for lba in 0..2048 {
    ftl.set(lba, lba * 12345); // Some pattern
  }

  // Force flush
  ftl.flush();

  // Verify LBA 1917
  let expected = 1917 * 12345;
  let got = ftl.get(1917);

  println!("LBA 1917: expected={}, got={:?}", expected, got);

  if got != Some(expected) {
    println!("MISMATCH!");
  } else {
    println!("OK");
  }
}
