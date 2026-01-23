use jdb_ftl_example::{Result, load_trace_map, save_svg_json};

fn main() -> Result<()> {
  println!("Loading trace and counting unique keys...");
  let map = load_trace_map()?;

  let unique_keys = map.len();
  println!("-----------------------------------------");
  println!("Total unique keys (unique LBAs): {}", unique_keys);

  #[derive(serde::Serialize)]
  struct Stats {
    valid_entries: u64,
    unique_keys: usize,
    locality: f64,
  }

  let stats = Stats {
    valid_entries: unique_keys as u64,
    unique_keys,
    locality: 1.0,
  };

  save_svg_json("key_stats", &stats)?;

  Ok(())
}
