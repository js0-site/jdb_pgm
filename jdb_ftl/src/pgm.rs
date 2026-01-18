use crate::{CompressionMode, GroupHeader};

pub struct Encoder;

impl Encoder {
  pub fn encode(group: &[u64]) -> GroupHeader {
    // 1. Check Constant
    // 2. Check Linear
    // 3. Fallback to Packed (calculate min/max, slope, build residual stream)
    
    // Placeholder implementation returning Constant 0
    GroupHeader {
      base: 0,
      slope: 0,
      offset: 0,
      config: (0 << 7) | (CompressionMode::Constant as u32),
    }
  }
}

pub struct Decoder;

impl Decoder {
   // TODO: Implement read logic
}
