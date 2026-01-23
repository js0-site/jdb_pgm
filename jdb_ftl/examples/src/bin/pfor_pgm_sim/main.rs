use std::{
  collections::HashMap,
  fs::File,
  io::{self, BufReader, Read},
  path::PathBuf,
};

use humansize::{BINARY, format_size};

/// Operation type: Write
const OP_WRITE: u8 = 1;
const RECORD_SIZE: usize = 16;
const GROUP_SIZE: usize = 4096;
const SKIP_INTERVAL: usize = 128;
const SEG_META_SIZE: usize = 12;

/// 精简方案：4 字节头部
const GROUP_HEADER_SIZE: usize = 4;

fn find_longest_segment(values: &[u64], epsilon: u64) -> (u64, i32, usize) {
  if values.is_empty() {
    return (0, 0, 0);
  }
  if values.len() == 1 {
    return (values[0], 0, 1);
  }
  let v0 = values[0] as i128;
  let eps = epsilon as i128;
  let (mut min_num, mut min_den) = (i128::MIN, 1i128);
  let (mut max_num, mut max_den) = (i128::MAX, 1i128);
  let mut best_len = 1;
  for (i, &val) in values.iter().enumerate().skip(1) {
    let x = i as i128;
    let y = val as i128;
    let (cl, ch) = (y - v0 - eps, y - v0 + eps);
    if min_num == i128::MIN || cl * min_den > min_num * x {
      min_num = cl;
      min_den = x;
    }
    if max_num == i128::MAX || ch * max_den < max_num * x {
      max_num = ch;
      max_den = x;
    }
    if min_num * max_den > max_num * min_den {
      break;
    }
    best_len = i + 1;
  }
  let slope = if best_len > 1 {
    let avg = ((min_num << 24) / min_den + (max_num << 24) / max_den) / 2;
    avg.clamp(i32::MIN as i128, i32::MAX as i128) as i32
  } else {
    0
  };
  let mut min_diff = i64::MAX;
  let mut acc = 0i64;
  for &val in values.iter().take(best_len) {
    let diff = (val as i64).wrapping_sub((v0 as i64).wrapping_add(acc >> 24));
    if diff < min_diff {
      min_diff = diff;
    }
    acc += slope as i64;
  }
  (
    (values[0] as i64).wrapping_add(min_diff) as u64,
    slope,
    best_len,
  )
}

fn calc_bit_width(values: &[u64], base: u64, slope: i32) -> u8 {
  let mut max_res = 0u64;
  for (i, &v) in values.iter().enumerate() {
    let pred = base.wrapping_add(((i as i64).wrapping_mul(slope as i64) >> 24) as u64);
    max_res = max_res.max(v.wrapping_sub(pred));
  }
  if max_res == 0 {
    0
  } else {
    (64 - max_res.leading_zeros()) as u8
  }
}

fn ef_byte_len(n: usize, u_bound: usize) -> usize {
  if n == 0 {
    return 1;
  }
  let l = if u_bound > n {
    (u_bound as f64 / n as f64).log2().floor() as usize
  } else {
    0
  };
  let upper_len_bits = n + (u_bound >> l) + 1;
  3 + upper_len_bits.div_ceil(8) + (n * l).div_ceil(8) + n.div_ceil(SKIP_INTERVAL) * 4
}

/// 真正的 PGM 拟合（无异常点）
fn pgm_pure_bits(values: &[u64], epsilon: u64) -> usize {
  if values.is_empty() {
    return 0;
  }
  let (mut cursor, mut total_bits, mut seg_count) = (0, 0, 0);
  while cursor < values.len() {
    let (base, slope, len) = find_longest_segment(&values[cursor..], epsilon);
    let bw = calc_bit_width(&values[cursor..cursor + len], base, slope);
    total_bits += (SEG_META_SIZE * 8) + (len * bw as usize);
    seg_count += 1;
    cursor += len;
  }
  total_bits + (ef_byte_len(seg_count, GROUP_SIZE) * 8)
}

/// 计算异常点最优存储位宽（每个分组独立）
fn calc_outlier_bits(pbas: &[u64], preds: &[u64]) -> usize {
  if pbas.is_empty() {
    return 0;
  }
  let mut max_diff = 0u64;
  for (p, pred) in pbas.iter().zip(preds) {
    let diff = if *p >= *pred { p - pred } else { pred - p };
    max_diff = max_diff.max(diff * 2); // ZigZag-ish
  }
  let bw = if max_diff == 0 {
    0
  } else {
    64 - max_diff.leading_zeros()
  } as usize;
  // 成本 = N * (16 bit 索引 + bw bit 残差)
  pbas.len() * (16 + bw)
}

fn main() -> io::Result<()> {
  let bin = std::env::var("BIN").unwrap_or_else(|_| "full".to_string());
  let path = PathBuf::from(format!("data/{}.bin", bin));
  println!("扫描 Trace...");
  let file = File::open(&path)?;
  let mut reader = BufReader::new(&file);
  let mut group_maps: HashMap<usize, HashMap<u16, u64>> = HashMap::new();
  let mut buf = [0u8; RECORD_SIZE];
  while reader.read_exact(&mut buf).is_ok() {
    let lba = u64::from_le_bytes(buf[0..8].try_into().unwrap());
    let meta = u64::from_le_bytes(buf[8..16].try_into().unwrap());
    if (meta >> 60) as u8 == OP_WRITE {
      group_maps
        .entry((lba / GROUP_SIZE as u64) as usize)
        .or_default()
        .insert((lba % GROUP_SIZE as u64) as u16, meta & 0x0FFFFFFFFFFFFFFF);
    }
  }
  let groups: Vec<Vec<u64>> = group_maps
    .into_values()
    .map(|map| {
      let mut e: Vec<_> = map.into_iter().collect();
      e.sort_unstable_by_key(|(o, _)| *o);
      e.into_iter().map(|(_, p)| p).collect()
    })
    .collect();

  println!("总组数: {}", groups.len());
  println!("\n[Residual-Patch 方案全局参数扫描]");
  println!(
    "{:>10} | {:>10} | {:>10} | {:>10} | {:>12} | {:>10}",
    "EPSILON", "跳过阈值", "Segments", "异常点", "总字节", "节省"
  );
  println!("{}", "-".repeat(85));

  let mut baseline_bytes = 0;
  for g in &groups {
    baseline_bytes += 4 + pgm_pure_bits(g, 512).div_ceil(8);
  }

  let mut best_saved = -100.0;
  let mut best_params = (0, 0);

  // 1. Sweep EPSILON
  for eps in [512, 1024, 4096, 16384] {
    // 2. Sweep Skip Threshold (多少个点以下就转为异常点更划算)
    for skip_thresh in [2, 4, 8, 16, 32] {
      let (mut total_bits, mut total_segs, mut total_outliers) = (0, 0, 0);
      for g in &groups {
        let (pgm_bits, segs, outliers, out_bits) = greedy_pfor_pgm_tuned(g, eps, skip_thresh);
        let group_bytes = GROUP_HEADER_SIZE + (pgm_bits + out_bits).div_ceil(8);
        let raw_bytes = 4 + g.len() * 8;
        if group_bytes > raw_bytes {
          total_bits += raw_bytes * 8;
          total_segs += 1;
        } else {
          total_bits += group_bytes * 8;
          total_segs += segs;
          total_outliers += outliers;
        }
      }
      let total_bytes = total_bits.div_ceil(8);
      let saved =
        (baseline_bytes as i64 - total_bytes as i64) as f64 / baseline_bytes as f64 * 100.0;

      if saved > best_saved {
        best_saved = saved;
        best_params = (eps, skip_thresh);
      }

      println!(
        "{:>10} | {:>10} | {:>10} | {:>10} | {:>12} | {:>+9.2}%",
        eps,
        skip_thresh,
        total_segs,
        total_outliers,
        format_size(total_bytes as u64, BINARY),
        saved
      );
    }
  }

  println!("\n========== 全局最优配置 ==========");
  println!("最优 EPSILON: {}", best_params.0);
  println!("最优跳过阈值: {}", best_params.1);
  println!("最高节省率: {:.2}%", best_saved);

  Ok(())
}

fn greedy_pfor_pgm_tuned(
  values: &[u64],
  epsilon: u64,
  skip_thresh: usize,
) -> (usize, usize, usize, usize) {
  if values.is_empty() {
    return (0, 0, 0, 0);
  }
  let mut cursor = 0;
  let mut total_residual_bits = 0usize;
  let mut seg_count = 0;
  let mut outlier_pbas = Vec::new();
  let mut outlier_preds = Vec::new();

  while cursor < values.len() {
    let (base, slope, len) = find_longest_segment(&values[cursor..], epsilon);

    // 如果拟合的长度不到阈值，且后面还有数据，就转为异常点
    if len < skip_thresh && cursor + len < values.len() {
      // 预估一个 pred
      let pred = base;
      outlier_pbas.push(values[cursor]);
      outlier_preds.push(pred);
      cursor += 1;
      continue;
    }

    let bw = calc_bit_width(&values[cursor..cursor + len], base, slope);
    total_residual_bits += len * bw as usize;
    seg_count += 1;
    cursor += len;
  }

  let pgm_bits = (seg_count * SEG_META_SIZE * 8)
    + (ef_byte_len(seg_count, GROUP_SIZE) * 8)
    + total_residual_bits;
  let outlier_bits = calc_outlier_bits(&outlier_pbas, &outlier_preds);

  (pgm_bits, seg_count, outlier_pbas.len(), outlier_bits)
}
