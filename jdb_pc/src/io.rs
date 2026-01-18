use super::{PcBase, types::BlockMeta};

/// Serialize Pc to bytes
/// 序列化 Pc 为字节流
pub fn dump<const B: usize>(pc: &PcBase<B>) -> Vec<u8> {
  let mut out = Vec::with_capacity(pc.size_in_bytes());
  out.extend_from_slice(&(pc.len as u64).to_le_bytes());

  // Helper macro for vector serialization
  macro_rules! serialize_vec {
    ($vec:expr, $serializer:expr) => {
      out.extend_from_slice(&($vec.len() as u32).to_le_bytes());
      for item in &$vec {
        $serializer(item);
      }
    };
  }

  serialize_vec!(pc.block_meta, |b: &BlockMeta| {
    out.extend_from_slice(&b.bit_offset.to_le_bytes());
    out.push(b.bit_width);
    out.push(b.flags);
    out.extend_from_slice(&b.exception_offset.to_le_bytes());
    // New fields
    out.extend_from_slice(&b.slope_fp.to_le_bytes());
    out.extend_from_slice(&b.intercept_fp.to_le_bytes());
  });

  // Optimized serialization for u64 vectors (bulk copy on little endian)
  // 针对 u64 向量的优化序列化（小端序批量复制）
  if cfg!(target_endian = "little") {
    let mut serialize_u64_fast = |vec: &[u64]| {
      out.extend_from_slice(&(vec.len() as u32).to_le_bytes());
      // SAFETY: u64 slice cast to u8 slice is safe for POD.
      // On LE, memory layout matches.
      unsafe {
        let ptr = vec.as_ptr() as *const u8;
        let len = vec.len() * 8;
        let slice = std::slice::from_raw_parts(ptr, len);
        out.extend_from_slice(slice);
      }
    };
    serialize_u64_fast(&pc.residuals);
    serialize_u64_fast(&pc.exceptions);
    serialize_u64_fast(&pc.bitmap);
  } else {
    serialize_vec!(pc.residuals, |r: &u64| out
      .extend_from_slice(&r.to_le_bytes()));
    serialize_vec!(pc.exceptions, |e: &u64| out
      .extend_from_slice(&e.to_le_bytes()));
    serialize_vec!(pc.bitmap, |b: &u64| out.extend_from_slice(&b.to_le_bytes()));
  }

  out
}

/// Deserialize Pc from bytes.
/// 从字节流反序列化 Pc。
pub fn load<const B: usize>(bytes: &[u8]) -> jdb_pgm_lib::error::Result<PcBase<B>> {
  let mut pos = 0;

  if bytes.len() < 8 {
    return Err(jdb_pgm_lib::error::PgmError::InvalidData(
      "Data too short for length header".into(),
    ));
  }
  let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
  pos += 8;

  macro_rules! check_len {
    ($needed:expr) => {
      if pos + $needed > bytes.len() {
        return Err(jdb_pgm_lib::error::PgmError::InvalidData(format!(
          "Unexpected EOF at pos {}, needed {}",
          pos, $needed
        )));
      }
    };
  }

  macro_rules! read_slice {
    ($len:expr) => {{
      check_len!($len);
      let slice = &bytes[pos..pos + $len];
      pos += $len;
      slice
    }};
  }

  macro_rules! read_u32 {
    () => {{ u32::from_le_bytes(read_slice!(4).try_into().unwrap()) }};
  }

  macro_rules! read_u64 {
    () => {{ u64::from_le_bytes(read_slice!(8).try_into().unwrap()) }};
  }

  macro_rules! deserialize_vec {
    ($deserializer:expr) => {{
      let count = read_u32!() as usize;
      let mut vec = Vec::with_capacity(count);
      for _ in 0..count {
        vec.push($deserializer()?);
      }
      vec
    }};
  }

  let block_meta = deserialize_vec!(|| -> jdb_pgm_lib::error::Result<BlockMeta> {
    let bit_offset = read_u32!();
    check_len!(2); // bit_width + flags
    let bit_width = bytes[pos];
    let flags = bytes[pos + 1];
    pos += 2;
    let exception_offset = u32::from_le_bytes(read_slice!(4).try_into().unwrap());
    let slope_fp = read_u64!();
    let intercept_fp = i64::from_le_bytes(read_slice!(8).try_into().unwrap());

    Ok(BlockMeta {
      bit_offset,
      bit_width,
      flags,
      exception_offset,
      slope_fp,
      intercept_fp,
    })
  });

  let load_u64_vec = |pos: &mut usize| -> jdb_pgm_lib::error::Result<Vec<u64>> {
    let p = *pos;
    if p + 4 > bytes.len() {
      return Err(jdb_pgm_lib::error::PgmError::InvalidData(format!(
        "EOF reading vec len at {}",
        p
      )));
    }
    let count = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as usize;
    *pos += 4;

    // Fast path for LE
    if cfg!(target_endian = "little") {
      let byte_len = count * 8;
      if *pos + byte_len > bytes.len() {
        return Err(jdb_pgm_lib::error::PgmError::InvalidData(format!(
          "EOF reading vec body at {}, needed {}",
          *pos, byte_len
        )));
      }
      let mut vec = Vec::with_capacity(count);
      #[allow(clippy::uninit_vec)]
      unsafe {
        vec.set_len(count);
        let src = bytes.as_ptr().add(*pos);
        let dst = vec.as_mut_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(src, dst, byte_len);
      }
      *pos += byte_len;
      Ok(vec)
    } else {
      let mut vec = Vec::with_capacity(count);
      for _ in 0..count {
        if *pos + 8 > bytes.len() {
          return Err(jdb_pgm_lib::error::PgmError::InvalidData(
            "EOF reading u64".into(),
          ));
        }
        let val = u64::from_le_bytes(bytes[*pos..*pos + 8].try_into().unwrap());
        *pos += 8;
        vec.push(val);
      }
      Ok(vec)
    }
  };

  let residuals = load_u64_vec(&mut pos)?;
  let exceptions = load_u64_vec(&mut pos)?;
  let bitmap = load_u64_vec(&mut pos)?;

  Ok(PcBase {
    block_meta,
    residuals,
    exceptions,
    bitmap,
    len,
  })
}
