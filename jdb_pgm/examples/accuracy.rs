use std::{collections::BTreeMap, fs::File, io::Write};

use jdb_pgm::Pgm;
use pgm_index as external_pgm;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rapidhash::RapidHashMap as HashMap;
use serde_json::json;
use tikv_jemalloc_ctl::{epoch, stats};
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

fn get_allocated() -> usize {
  epoch::advance().unwrap();
  stats::allocated::read().unwrap()
}

fn main() {
  let size = 1_000_000;
  let epsilons = vec![32, 64, 128];

  // Use random gaps distribution for realistic accuracy measurement
  let mut rng = StdRng::seed_from_u64(42);
  let mut cur = 0u64;
  let data: Vec<u64> = (0..size)
    .map(|_| {
      cur += rng.random_range(1..100);
      cur
    })
    .collect();
  let actual_size = data.len();
  let _data_bytes = actual_size * 8;

  let mut results = Vec::new();

  for &eps in &epsilons {
    // 1. jdb_pgm
    let pgm = Pgm::new(&data, eps);
    let mut max_err = 0;
    let mut total_err = 0u64;
    for (i, &key) in data.iter().enumerate() {
      let pred = pgm.predict(key);
      let err = (pred as isize - i as isize).unsigned_abs() as u64;
      if err > max_err {
        max_err = err;
      }
      total_err += err;
    }
    let avg_err = total_err as f64 / actual_size as f64;

    results.push(json!({
        "group": "accuracy",
        "algorithm": "jdb_pgm",
        "epsilon": eps,
        "data_size": actual_size,
        "max_error": max_err,
        "avg_error": avg_err,
        "memory_bytes": pgm.mem_usage()
    }));

    // 2. external_pgm
    let ext = external_pgm::PGMIndex::new(data.clone(), eps);
    let mut max_err = 0;
    let mut total_err = 0u64;
    for (i, &key) in data.iter().enumerate() {
      let pred = ext.predict_pos(key);
      let err = (pred as isize - i as isize).unsigned_abs() as u64;
      if err > max_err {
        max_err = err;
      }
      total_err += err;
    }
    let avg_err = total_err as f64 / actual_size as f64;

    results.push(json!({
        "group": "accuracy",
        "algorithm": "external_pgm",
        "epsilon": eps,
        "data_size": actual_size,
        "max_error": max_err,
        "avg_error": avg_err,
        "memory_bytes": ext.memory_usage().saturating_sub(actual_size * 8)
    }));
  }

  // 3. HashMap
  let start_mem = get_allocated();
  let map: HashMap<u64, usize> = data.iter().enumerate().map(|(i, &v)| (v, i)).collect();
  let end_mem = get_allocated();
  results.push(json!({
      "group": "accuracy",
      "algorithm": "hashmap",
      "data_size": actual_size,
      "memory_bytes": end_mem.saturating_sub(start_mem)
  }));
  drop(map);

  // 4. BTreeMap
  let start_mem = get_allocated();
  let btree: BTreeMap<u64, usize> = data.iter().enumerate().map(|(i, &v)| (v, i)).collect();
  let end_mem = get_allocated();
  results.push(json!({
      "group": "accuracy",
      "algorithm": "btreemap",
      "data_size": actual_size,
      "memory_bytes": end_mem.saturating_sub(start_mem)
  }));
  drop(btree);

  // 5. Binary Search (Just data)
  results.push(json!({
      "group": "accuracy",
      "algorithm": "binary_search",
      "data_size": actual_size,
      "memory_bytes": 0
  }));

  let final_json = json!({ "results": results });
  let mut file = File::create("/tmp/jdb_pgm_accuracy.json").unwrap();
  file.write_all(final_json.to_string().as_bytes()).unwrap();
  println!("Accuracy and memory data saved to /tmp/jdb_pgm_accuracy.json");
}
