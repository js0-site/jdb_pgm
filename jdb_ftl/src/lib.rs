mod pgm;

pub use pgm::Encoder;

pub const PAGE_SIZE: usize = 4096;
pub const HEADER_SIZE: usize = 16;
pub const GROUPS_PER_FRAME: usize = 16;
pub const ENTRIES_PER_GROUP: usize = 32;
pub const ENTRIES_PER_FRAME: usize = GROUPS_PER_FRAME * ENTRIES_PER_GROUP; // 512

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionMode {
  Constant = 0,
  Linear = 1,
  Packed = 2,
  Exception = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct GroupHeader {
  pub base: u64,
  pub slope: i16,
  pub offset: u16,
  pub config: u32,
}

impl GroupHeader {
  pub fn width(&self) -> u8 {
    (self.config & 0x7F) as u8
  }

  pub fn mode(&self) -> CompressionMode {
    match (self.config >> 7) & 0x3 {
      0 => CompressionMode::Constant,
      1 => CompressionMode::Linear,
      2 => CompressionMode::Packed,
      3 => CompressionMode::Exception,
      _ => unreachable!(),
    }
  }

  pub fn exception_map(&self) -> u32 {
    self.config >> 10
  }
}

pub struct Frame {
  pub headers: [GroupHeader; GROUPS_PER_FRAME],
  pub payload: Vec<u8>,
}

impl Default for Frame {
  fn default() -> Self {
    Self {
      headers: [GroupHeader {
        base: 0,
        slope: 0,
        offset: 0,
        config: 0,
      }; GROUPS_PER_FRAME],
      payload: Vec::with_capacity(PAGE_SIZE - 256),
    }
  }
}

impl Frame {
  pub fn get(&self, idx: usize) -> u64 {
    let group_idx = idx / ENTRIES_PER_GROUP;
    let sub_idx = idx % ENTRIES_PER_GROUP;
    
    // Prediction
    let h = &self.headers[group_idx];
    let pred = h.base.wrapping_add((sub_idx as u64).wrapping_mul(h.slope as u64));
    
    match h.mode() {
       CompressionMode::Constant | CompressionMode::Linear => pred,
       _ => {
         // TODO: Implement Read Path in pgm.rs and call it here, or implement inline
         // For now just return prediction to compile
         pred
       }
    }
  }
}
