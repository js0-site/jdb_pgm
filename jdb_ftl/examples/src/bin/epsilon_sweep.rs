use jdb_ftl_example::{Result, load_trace_map, map_to_sorted_vec, save_svg_json};
use serde::Serialize;

#[derive(Serialize)]
struct Point {
  x: usize,
  y: usize,
}

fn main() -> Result<()> {
  println!("Epsilon Sensitivity Sweep / 精度灵敏度扫描");

  let map = load_trace_map()?;
  let data = map_to_sorted_vec(&map);
  println!("Unique entries: {}", data.len());

  let epsilons = [1, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
  let mut results = Vec::with_capacity(epsilons.len());

  for &eps in &epsilons {
    let count = estimate_total_segments(&data, eps);
    println!("ε = {:4} => Segments: {}", eps, count);
    results.push(Point { x: eps, y: count });
  }

  save_svg_json("epsilon_sweep", &results)?;
  Ok(())
}

fn estimate_total_segments(data: &[(u64, u64)], eps: usize) -> usize {
  const GROUP_SIZE: u64 = 4096;
  if data.is_empty() {
    return 0;
  }

  let mut total = 0;
  let mut i = 0;
  while i < data.len() {
    let group_id = data[i].0 / GROUP_SIZE;
    let mut j = i + 1;
    while j < data.len() && data[j].0 / GROUP_SIZE == group_id {
      j += 1;
    }
    total += estimate_segments(&data[i..j], eps);
    i = j;
  }
  total
}

/// Estimate segments using greedy PGM approach
/// 使用贪心 PGM 方法估算段数
fn estimate_segments(data: &[(u64, u64)], eps: usize) -> usize {
  let len = data.len();
  if len == 0 {
    return 0;
  }
  if len <= 8 {
    return 1;
  }

  let mut segs = 0;
  let mut i = 0;
  while i < len {
    let (s_lba, s_pba) = data[i];
    let s_pba = s_pba as i64;

    let slope = if i + 1 < len {
      let dl = (data[i + 1].0 - s_lba) as i64;
      let dp = data[i + 1].1 as i64 - s_pba;
      if dl > 0 { (dp << 24) / dl } else { 0 }
    } else {
      0
    };

    let mut j = i;
    while j < len {
      let dx = (data[j].0 - s_lba) as i64;
      let pred = s_pba + ((slope * dx) >> 24);
      let error = (data[j].1 as i64 - pred).unsigned_abs() as usize;
      if error > eps * 2 {
        break;
      }
      j += 1;
    }
    segs += 1;
    i = if j > i { j } else { i + 1 };
  }
  segs
}
