use super::{traits::*, util::*, view::EfView};

pub struct EfIter<'a, L: EfLayout> {
  view: EfView<'a, L>,
  idx: usize,
  curr_high: u64,

  upper_bit_pos: usize,
  upper_word_cache: u64,
  upper_cache_bits: usize,
}

impl<'a, L: EfLayout> EfIter<'a, L> {
  pub fn new(view: EfView<'a, L>) -> Self {
    Self {
      view,
      idx: 0,
      curr_high: 0,
      upper_bit_pos: 0,
      upper_word_cache: 0,
      upper_cache_bits: 0,
    }
  }
}

impl<'a, L: EfLayout> Iterator for EfIter<'a, L> {
  type Item = L::Primitive;

  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    if self.idx >= self.view.n {
      return None;
    }

    loop {
      if self.upper_cache_bits == 0 && !self.refill_upper_safe() {
        return None;
      }

      let zeros = self.upper_word_cache.trailing_zeros() as usize;

      if zeros < self.upper_cache_bits {
        self.curr_high += zeros as u64;

        let bits_consumed = zeros + 1;
        if bits_consumed >= 64 {
          self.upper_word_cache = 0;
          self.upper_cache_bits = 0;
        } else {
          self.upper_word_cache >>= bits_consumed;
          self.upper_cache_bits -= bits_consumed;
        }
        self.upper_bit_pos += bits_consumed;

        break;
      } else {
        self.curr_high += self.upper_cache_bits as u64;
        self.upper_bit_pos += self.upper_cache_bits;
        self.upper_cache_bits = 0;
      }
    }

    let l = self.view.l;
    let lower_val = if l == 0 {
      0
    } else {
      read_bits_u64_at(self.view.data, self.view.lower_offset, self.idx * l, l)
    };

    let val_u64 = (self.curr_high << l) | lower_val;
    self.idx += 1;
    Some(L::Primitive::from_u64(val_u64))
  }
}

impl<'a, L: EfLayout> EfIter<'a, L> {
  #[inline(always)]
  fn refill_upper_safe(&mut self) -> bool {
    if self.upper_bit_pos >= self.view.upper_len_bits {
      return false;
    }
    let byte_idx = self.view.upper_offset + (self.upper_bit_pos / 8);
    let bit_offset = self.upper_bit_pos % 8;

    let raw = if byte_idx + 8 <= self.view.data.len() {
      unsafe {
        u64::from_le((self.view.data.as_ptr().add(byte_idx) as *const u64).read_unaligned())
      }
    } else {
      load_u64_safe(self.view.data, byte_idx)
    };

    self.upper_word_cache = raw >> bit_offset;
    self.upper_cache_bits = 64 - bit_offset;
    true
  }
}
