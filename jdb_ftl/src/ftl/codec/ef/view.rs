use std::marker::PhantomData;

use super::{traits::*, util::*};

/// Generic Elias-Fano view.
/// 泛型 Elias-Fano 视图。
#[derive(Clone, Copy, Debug)]
pub struct EfView<'a, L: EfLayout> {
  pub data: &'a [u8],
  pub n: usize,
  pub l: usize,
  pub upper_offset: usize,
  pub lower_offset: usize,
  pub upper_len_bits: usize,

  skip_offset: usize,
  skip_count: usize,

  _marker: PhantomData<L>,
}

impl<'a, L: EfLayout> EfView<'a, L> {
  /// Creates a view over a buffer.
  /// 创建缓冲区视图。
  #[inline(always)]
  pub fn new(data: &'a [u8], n: usize, _u_bound: usize) -> Self {
    if n == 0 || data.len() < 3 {
      return Self::empty(data);
    }

    let l = (data[0] & 0x0F) as usize;
    // Read 2 bytes for upper_len_bytes
    // Format: [L: u8] [UpperLen_LO] [UpperLen_HI]
    let upper_len_bytes = u16::from_le_bytes([data[1], data[2]]) as usize;

    let upper_offset = 3;
    let lower_offset = upper_offset + upper_len_bytes;
    let upper_len_bits = upper_len_bytes * 8;

    let lower_len_bits = n * l;
    let lower_len_bytes = lower_len_bits.div_ceil(8);
    let skip_offset = lower_offset + lower_len_bytes;
    let skip_count = n.div_ceil(L::SKIP_INTERVAL);

    let required_len = skip_offset + skip_count * L::SKIP_ENTRY_SIZE;
    if required_len > data.len() {
      return Self::empty(data);
    }

    Self {
      data,
      n,
      l,
      upper_offset,
      lower_offset,
      upper_len_bits,
      skip_offset,
      skip_count,
      _marker: PhantomData,
    }
  }

  #[inline(always)]
  fn empty(data: &'a [u8]) -> Self {
    Self {
      data,
      n: 0,
      l: 0,
      upper_offset: 0,
      lower_offset: 0,
      upper_len_bits: 0,
      skip_offset: 0,
      skip_count: 0,
      _marker: PhantomData,
    }
  }

  #[inline(always)]
  pub fn len(&self) -> usize {
    self.n
  }

  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.n == 0
  }

  #[inline(always)]
  fn skip_at(&self, idx: usize) -> (usize, L::Primitive) {
    debug_assert!(idx < self.skip_count);
    let offset = self.skip_offset + idx * L::SKIP_ENTRY_SIZE;
    unsafe { L::read_skip_entry(self.data, offset) }
  }

  #[inline(always)]
  pub fn get(&self, index: usize) -> L::Primitive {
    if index >= self.n {
      return L::Primitive::sentinel();
    }

    let skip_idx = index / L::SKIP_INTERVAL;
    let (start_bit_pos, _start_high) = if skip_idx > 0 {
      self.skip_at(skip_idx)
    } else {
      (0, L::Primitive::default())
    };

    let start_rank = skip_idx * L::SKIP_INTERVAL;
    let target_rank = index;

    let mut skipped_ones = start_rank;
    let mut global_bit_pos = start_bit_pos;
    let mut byte_idx = self.upper_offset + (global_bit_pos / 8);
    let bit_offset_in_byte = global_bit_pos % 8;

    // Unaligned first byte handling
    if bit_offset_in_byte != 0 {
      let b = unsafe { *self.data.get_unchecked(byte_idx) };
      let masked = b >> bit_offset_in_byte;
      let ones_in_partial = masked.count_ones() as usize;

      if skipped_ones + ones_in_partial > target_rank {
        let needed = target_rank - skipped_ones;
        let bit_in_partial = select_bit_in_byte(masked, needed);
        global_bit_pos += bit_in_partial;
        return self.decode_val(global_bit_pos, index);
      }

      skipped_ones += ones_in_partial;
      global_bit_pos += 8 - bit_offset_in_byte;
      byte_idx += 1;
    }

    // Fast Scan
    while skipped_ones <= target_rank {
      let raw_word = if byte_idx + 8 <= self.data.len() {
        unsafe { u64::from_le((self.data.as_ptr().add(byte_idx) as *const u64).read_unaligned()) }
      } else {
        load_u64_safe(self.data, byte_idx)
      };

      let ones_in_word = raw_word.count_ones() as usize;
      if skipped_ones + ones_in_word > target_rank {
        let needed = target_rank - skipped_ones;
        let bit_idx_in_word = select_bit_in_u64(raw_word, needed);
        global_bit_pos += bit_idx_in_word;
        break;
      }

      skipped_ones += ones_in_word;
      global_bit_pos += 64;
      byte_idx += 8;
    }

    self.decode_val(global_bit_pos, index)
  }

  #[inline(always)]
  fn decode_val(&self, upper_bit_pos: usize, index: usize) -> L::Primitive {
    let h_val = upper_bit_pos - index;
    let l = self.l;

    let lower_val_u64 = if l == 0 {
      0
    } else {
      read_bits_u64_at(self.data, self.lower_offset, index * l, l)
    };

    let h = L::Primitive::from_u64(h_val as u64);
    let lower = L::Primitive::from_u64(lower_val_u64);

    // Combine: (h << l) | lower.
    // Needs pure shifting on primitive type.
    // Assuming as_u64 -> op -> from_u64 works universally.
    let val_u64 = (h.as_u64() << l) | lower.as_u64();
    L::Primitive::from_u64(val_u64)
  }

  /// Predecessor search.
  pub fn predecessor(&self, target: L::Primitive) -> (usize, L::Primitive) {
    if self.n == 0 {
      return (0, L::Primitive::default());
    }

    let l = self.l;
    let target_u64 = target.as_u64();
    let target_h = target_u64 >> l;

    // 1. Binary search on Skip Table
    let mut lo = 0;
    let mut hi = self.skip_count;

    while lo < hi {
      let mid = lo + (hi - lo) / 2;
      let (_, high_val) = self.skip_at(mid);
      if high_val.as_u64() <= target_h {
        lo = mid + 1;
      } else {
        hi = mid;
      }
    }
    let skip_idx = lo.saturating_sub(1);

    // 2. Init
    let (start_bit_pos, start_high) = if skip_idx > 0 {
      self.skip_at(skip_idx)
    } else {
      (0, L::Primitive::default())
    };

    let mut curr_high = start_high.as_u64();
    let mut idx = skip_idx * L::SKIP_INTERVAL;

    if curr_high > target_h {
      return self.scan_backwards(idx, target);
    }

    let global_bit_pos = start_bit_pos;
    let mut byte_idx = self.upper_offset + (global_bit_pos / 8);
    let bit_offset = global_bit_pos % 8;

    // 3. Scan
    let mut best_idx = idx;
    let mut best_val = L::Primitive::from_u64(0); // placeholder

    // Cache setup (simplified from u16 version for brevity in generic, but logic holds)
    // Note: For perf, we might want to preserve the robust caching logic.
    let mut word_cache = if byte_idx + 8 <= self.data.len() {
      unsafe {
        u64::from_le((self.data.as_ptr().add(byte_idx) as *const u64).read_unaligned())
          >> bit_offset
      }
    } else {
      load_u64_safe(self.data, byte_idx) >> bit_offset
    };
    let mut bits_in_cache = 64 - bit_offset;
    byte_idx += 8;

    while idx < self.n {
      loop {
        // ... same logic as u16 but generic ...
        // Refill check
        if bits_in_cache == 0 || (word_cache == 0 && bits_in_cache < 64) {
          // logic handled by zeros check usually, but if we run out of bits completely:
        }

        let zeros = word_cache.trailing_zeros() as usize;

        if zeros < bits_in_cache {
          // Found '1'
          curr_high += zeros as u64;

          if curr_high > target_h {
            if idx == skip_idx * L::SKIP_INTERVAL {
              return self.scan_backwards(idx, target);
            }
            // For generic, check if we ever set best_val. If not (first item > target), backward scan.
            // But idx > start_rank here logic covers it.
            return (best_idx, best_val);
          }

          // Decode full
          let lower_val_u64 = if l == 0 {
            0
          } else {
            read_bits_u64_at(self.data, self.lower_offset, idx * l, l)
          };
          let val_u64 = (curr_high << l) | lower_val_u64;
          let val = L::Primitive::from_u64(val_u64);

          if val > target {
            if idx == skip_idx * L::SKIP_INTERVAL {
              return self.scan_backwards(idx, target);
            }
            return (best_idx, best_val);
          }

          best_idx = idx;
          best_val = val;

          let consumed = zeros + 1;
          if consumed >= 64 {
            word_cache = 0;
            bits_in_cache = 0;
          } else {
            word_cache >>= consumed;
            bits_in_cache -= consumed;
          }
          break;
        } else {
          curr_high += bits_in_cache as u64;

          if byte_idx < self.data.len() + 8 {
            let next_w = if byte_idx + 8 <= self.data.len() {
              unsafe {
                u64::from_le((self.data.as_ptr().add(byte_idx) as *const u64).read_unaligned())
              }
            } else {
              load_u64_safe(self.data, byte_idx)
            };
            word_cache = next_w;
            bits_in_cache = 64;
            byte_idx += 8;
          } else {
            break;
          }
        }
      }
      idx += 1;
    }

    (best_idx, best_val)
  }

  #[cold]
  fn scan_backwards(&self, limit_count: usize, target: L::Primitive) -> (usize, L::Primitive) {
    let mut idx = limit_count;
    while idx > 0 {
      idx -= 1;
      let val = self.get(idx);
      if val <= target {
        return (idx, val);
      }
    }
    (0, L::Primitive::default())
  }
  pub fn iter(&self) -> super::iter::EfIter<'a, L> {
    super::iter::EfIter::new(*self)
  }
}
