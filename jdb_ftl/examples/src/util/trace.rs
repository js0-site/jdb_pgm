use std::{
  env,
  fs::File,
  io::{self, BufReader, Read},
};

use anyhow::{Context, Result};
use rapidhash::{HashMapExt, RapidHashMap};

use crate::util::path::data_dir;

/// Size of a trace record in bytes
/// Trace 记录的字节大小
pub const RECORD_SIZE: usize = 16;
/// Write operation code
/// 写操作代码
pub const OP_WRITE: u8 = 1;
/// Read operation code
/// 读操作代码
pub const OP_READ: u8 = 0;

/// A single trace record
/// 单条 Trace 记录
#[derive(Debug, Clone, Copy)]
pub struct OpRec {
  pub op: u8,
  pub lba: u64,
  pub pba: u64,
}

/// Iterator over trace records in a file
/// Trace 记录文件迭代器
pub struct TraceIter<R> {
  reader: R,
}

impl<R: Read> TraceIter<R> {
  pub fn new(reader: R) -> Self {
    Self { reader }
  }
}

impl<R: Read> Iterator for TraceIter<R> {
  type Item = io::Result<OpRec>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let mut buf = [0u8; RECORD_SIZE];
    match self.reader.read_exact(&mut buf) {
      Ok(()) => {
        let lba = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        let meta = u64::from_le_bytes(buf[8..16].try_into().unwrap());
        Some(Ok(OpRec {
          op: (meta >> 60) as u8,
          lba,
          pba: meta & 0x0FFFFFFFFFFFFFFF,
        }))
      }
      Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
      Err(e) => Some(Err(e)),
    }
  }
}

/// Load the trace file specified by BIN env var (defaults to "full")
/// 加载 BIN 环境变量指定的 Trace 文件（默认为 "full"）
pub fn open_bin() -> Result<(File, String)> {
  let name = env::var("BIN").unwrap_or_else(|_| "full".into());
  let data_path = data_dir();
  let path = data_path.join(format!("{}.name.bin", name));
  let path = if path.exists() {
    path
  } else {
    data_path.join(format!("{}.bin", name))
  };

  let file = File::open(&path).with_context(|| format!("Failed to open trace: {:?}", path))?;
  Ok((file, name))
}

/// Replay the trace and return a map of LBA -> last PBA
/// 重放 Trace 并返回 LBA -> 最后一次写入的 PBA 映射
pub fn load_trace_map() -> Result<RapidHashMap<u64, u64>> {
  let (file, _) = open_bin()?;
  let reader = BufReader::with_capacity(1024 * 1024 * 8, file);
  let mut map = RapidHashMap::new();

  for rec in TraceIter::new(reader) {
    let rec = rec?;
    if rec.op == OP_WRITE {
      if rec.pba == u64::MAX {
        map.remove(&rec.lba);
      } else {
        map.insert(rec.lba, rec.pba);
      }
    }
  }

  Ok(map)
}

/// Convert map to sorted vector of (LBA, PBA)
/// 将映射转换为排序后的 (LBA, PBA) 向量
pub fn map_to_sorted_vec(map: &RapidHashMap<u64, u64>) -> Vec<(u64, u64)> {
  let mut vec: Vec<_> = map.iter().map(|(&k, &v)| (k, v)).collect();
  vec.sort_unstable_by_key(|(lba, _)| *lba);
  vec
}
