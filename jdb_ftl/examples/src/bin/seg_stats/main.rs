use std::{
  env,
  io::{self, BufReader, Seek, SeekFrom, Write},
  time::Instant,
};

use jdb_ftl::{DefaultFtl, FtlTrait};
use jdb_ftl_example::{OP_WRITE, Result, TraceIter, open_bin, save_svg_json};

mod i18n;
use i18n::{En, I18n, Zh};

fn run<I: I18n>() -> Result<()> {
  let (file, _) = open_bin()?;
  let mut reader = BufReader::new(file);

  // 1. Scan trace for max LBA
  // 1. 扫描 trace 获取最大 LBA
  println!("{}", I::SCANNING_TRACE);
  let mut max_lba = 0u64;
  let mut op_count = 0usize;

  for res in TraceIter::new(&mut reader) {
    let rec = res?;
    if rec.lba > max_lba {
      max_lba = rec.lba;
    }
    op_count += 1;
  }
  max_lba += 1;

  println!(
    "{}",
    I::TRACE_SCANNED
      .replace("{}", &op_count.to_string())
      .replace("{}", &max_lba.to_string())
  );

  // 2. Initialize FTL
  // 2. 初始化 FTL
  println!("{}", I::INITIALIZING_FTL);
  let mut ftl = DefaultFtl::new(max_lba);

  // 3. Replay trace
  // 3. 重放 trace
  println!("{}", I::REPLAYING_TRACE);
  let start = Instant::now();
  reader.get_mut().seek(SeekFrom::Start(0))?;

  let mut processed = 0usize;
  let mut stdout = io::stdout();
  let mut lba_heatmap_100k = std::collections::HashMap::new();

  for res in TraceIter::new(reader) {
    let rec = res?;
    if rec.op == OP_WRITE {
      ftl.set(rec.lba, rec.pba);
      let bucket = (rec.lba / 100_000) as usize;
      *lba_heatmap_100k.entry(bucket).or_insert(0) += 1;
    }
    processed += 1;
    if processed.is_multiple_of(100_000) {
      print!(
        "\r{}",
        I::PROCESSED_OPS.replace("{}", &processed.to_string())
      );
      stdout.flush()?;
    }
  }
  println!(
    "\r{}",
    I::PROCESSED_OPS.replace("{}", &processed.to_string())
  );

  println!(
    "{}",
    I::REPLAY_DURATION.replace("{:.2?}", &format!("{:.2?}", start.elapsed()))
  );
  println!("{}", I::SYNCING_TASKS);
  ftl.flush();

  // 4. Inspect segments
  // 4. 检查 segments
  println!("{}", I::INSPECTING_SEGMENTS);
  let stats = ftl.inspect_all_segments();
  let total_groups = stats.group_count_pgm + stats.group_count_direct + stats.group_count_empty;

  println!(
    "{}",
    I::TOTAL_SEGMENTS.replace("{}", &total_groups.to_string())
  );
  println!(
    "{}",
    I::PGM_GROUPS.replace("{}", &stats.group_count_pgm.to_string())
  );
  println!(
    "{}",
    I::DIRECT_GROUPS.replace("{}", &stats.group_count_direct.to_string())
  );
  println!(
    "{}",
    I::EMPTY_GROUPS.replace("{}", &stats.group_count_empty.to_string())
  );

  let compression = if stats.total_bytes_logical > 0 {
    stats.total_bytes_logical as f64 / stats.total_bytes_physical as f64
  } else {
    1.0
  };
  println!(
    "{}",
    I::COMPRESSION_FACTOR
      .replace("{:.2}x", &format!("{:.2}x", compression))
      .replace(
        "{:.2}%",
        &format!("{:.2}%", (1.0 - 1.0 / compression) * 100.0)
      )
  );

  // Additional stats display...
  // Export to JSON for SVG generation
  let mut bit_width_dist = std::collections::HashMap::new();
  for &bw in &stats.segment_bit_widths {
    *bit_width_dist.entry(bw).or_insert(0) += 1;
  }

  let mut seg_len_dist = std::collections::HashMap::new();
  for &len in &stats.segment_lengths {
    let bucket = (len / 10) * 10;
    *seg_len_dist.entry(bucket).or_insert(0) += 1;
  }
  
  #[derive(serde::Serialize)]
  struct Groups {
    pgm: usize,
    direct: usize,
    empty: usize,
  }

  #[derive(serde::Serialize)]
  struct JsonStats {
    total_groups: usize,
    groups: Groups,
    total_pgm_segments: usize,
    sim_type_a: usize,
    sim_type_b: usize,
    saved_linear: usize,
    saved_payload: usize,
    saved_exception: usize,
    bit_width_dist: std::collections::HashMap<u8, usize>,
    seg_len_dist: std::collections::HashMap<usize, usize>,
    lba_heatmap_100k: std::collections::HashMap<usize, usize>,
  }

  let json_stats = JsonStats {
    total_groups,
    groups: Groups {
      pgm: stats.group_count_pgm,
      direct: stats.group_count_direct,
      empty: stats.group_count_empty,
    },
    total_pgm_segments: stats.segment_lengths.len(),
    sim_type_a: stats.type_a_count,
    sim_type_b: stats.type_b_count,
    saved_linear: stats.linear_model_bytes_saved,
    saved_payload: stats.payload_bytes_saved,
    saved_exception: stats.exception_table_bytes_saved,
    bit_width_dist,
    seg_len_dist,
    lba_heatmap_100k,
  };

  save_svg_json("stats", &json_stats)?;
  Ok(())
}

fn main() -> Result<()> {
  let lang = env::var("LANG").unwrap_or_default();
  if lang.starts_with("zh") {
    run::<Zh>()
  } else {
    run::<En>()
  }
}
