/// ZigZag encoding: maps signed integers to unsigned integers.
/// 0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3, 2 -> 4, ...
#[inline(always)]
pub fn zigzag_encode(val: i64) -> u64 {
  ((val << 1) ^ (val >> 63)) as u64
}

/// ZigZag decoding: maps unsigned integers back to signed integers.
#[inline(always)]
pub fn zigzag_decode(val: u64) -> i64 {
  ((val >> 1) as i64) ^ (-((val & 1) as i64))
}

/// Calculate the number of bits required to store a value.
#[inline(always)]
pub fn bit_width(val: u64) -> u8 {
  if val == 0 {
    0
  } else {
    (64 - val.leading_zeros()) as u8
  }
}
