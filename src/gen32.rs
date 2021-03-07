use core::convert::{TryFrom, TryInto};

/// A Generator with 32 bits of output per step.
pub trait Gen32 {
  /// Generates the next 32 bits of output.
  fn next_u32(&mut self) -> u32;

  /// Produce a `bool`
  #[inline(always)]
  fn next_bool(&mut self) -> bool {
    (self.next_u32() as i32) < 0
  }

  /// Produce a `u8`
  #[inline(always)]
  fn next_u8(&mut self) -> u8 {
    (self.next_u32() >> 24) as u8
  }

  /// Produce a `u16`
  #[inline(always)]
  fn next_u16(&mut self) -> u16 {
    (self.next_u32() >> 16) as u16
  }

  /// Produce a `u64`
  #[inline(always)]
  fn next_u64(&mut self) -> u64 {
    let l = self.next_u32() as u64;
    let h = self.next_u32() as u64;
    h << 32 | l
  }

  /// Returns an `f32` in the unsigned unit range, `[0, 1]`
  ///
  /// If you'd like `[0, 1)` then just use this and reroll in the (very
  /// unlikely) case that you do get 1.0
  #[inline]
  fn next_f32_unit(&mut self) -> f32 {
    crate::free_utils::ieee754_random_f32(self, true)
  }

  /// Returns an `f32` in the signed unit range, `[-1, 1]`
  #[inline]
  fn next_f32_signed_unit(&mut self) -> f32 {
    crate::free_utils::ieee754_random_f32(self, false)
  }

  /// Returns an `f64` in the unsigned unit range, `[0, 1]`
  ///
  /// If you'd like `[0, 1)` then just use this and reroll in the (very
  /// unlikely) case that you do get 1.0
  #[inline]
  fn next_f64_unit(&mut self) -> f64 {
    crate::free_utils::ieee754_random_f64(self, true)
  }

  /// Returns an `f64` in the signed unit range, `[-1, 1]`
  #[inline]
  fn next_f64_signed_unit(&mut self) -> f64 {
    crate::free_utils::ieee754_random_f64(self, false)
  }

  /// Gives a value within `0 .. B`
  ///
  /// This is often more efficient than making a
  /// [`BoundedRandU32`](crate::BoundedRandU32) if you don't need to use a
  /// specific bound value more than once.
  ///
  /// ## Panics
  /// * If the input is 0.
  #[inline]
  fn next_bounded(&mut self, b: u32) -> u32 {
    assert!(b != 0, "Gen32::next_bounded> Bound must be non-zero.");
    let mut x = self.next_u32() as u64;
    let mut mul = (b as u64).wrapping_mul(x);
    let mut low = mul as u32;
    if low < b {
      let threshold = b.wrapping_neg() % b;
      while low < threshold {
        x = self.next_u32() as u64;
        mul = (b as u64).wrapping_mul(x);
        low = mul as u32;
      }
    }
    let high = (mul >> 32) as u32;
    high
  }

  /// Gets a value out of the slice given (by copy).
  ///
  /// * The default impl will not pick past index `u32::MAX`.
  #[inline(always)]
  fn pick<T>(&mut self, buf: &[T]) -> T
  where
    Self: Sized,
    T: Copy,
  {
    let end: u32 = saturating_usize_as_u32(buf.len());
    buf[usize::try_from(self.next_bounded(end)).unwrap()]
  }

  /// Gets a value out of the slice given (by shared ref).
  ///
  /// * The default impl will not pick past index `u32::MAX`.
  #[inline(always)]
  fn pick_ref<'b, T>(&mut self, buf: &'b [T]) -> &'b T
  where
    Self: Sized,
  {
    let end: u32 = saturating_usize_as_u32(buf.len());
    &buf[usize::try_from(self.next_bounded(end)).unwrap()]
  }

  /// Gets a value out of the slice given (by unique ref).
  ///
  /// * The default impl will not pick past index `u32::MAX`.
  #[inline(always)]
  fn pick_mut<'b, T>(&mut self, buf: &'b mut [T]) -> &'b mut T
  where
    Self: Sized,
  {
    let end: u32 = saturating_usize_as_u32(buf.len());
    &mut buf[usize::try_from(self.next_bounded(end)).unwrap()]
  }

  /// Shuffles a slice in `O(len)` time.
  ///
  /// * The default impl shuffles only the first `u32::MAX` elements.
  #[inline]
  fn shuffle<T>(&mut self, buf: &mut [T])
  where
    Self: Sized,
  {
    // Note(Lokathor): The "standard" Fisher-Yates shuffle goes backward from
    // the end of the slice, but this version allows us to access memory forward
    // from the start to the end, so that we play more nicely with the
    // fetch-ahead of most modern CPUs.
    let mut possibility_count: u32 = buf.len().try_into().unwrap_or(u32::max_value());
    let mut this_index: usize = 0;
    let end = buf.len() - 1;
    while this_index < end {
      let offset = self.next_bounded(possibility_count) as usize;
      buf.swap(this_index, this_index + offset);
      possibility_count -= 1;
      this_index += 1;
    }
  }
}

// Asserts that `Gen32` is an object-safe trait.
const _: [&mut dyn Gen32; 0] = [];

/// Converts the `usize` into a `u32`, or gives `u32::MAX` if that wouldn't fit.
#[inline(always)]
const fn saturating_usize_as_u32(val: usize) -> u32 {
  #[cfg(target_pointer_width = "16")]
  {
    val as u32
  }
  #[cfg(target_pointer_width = "32")]
  {
    val as u32
  }
  #[cfg(target_pointer_width = "64")]
  {
    if val <= core::u32::MAX as usize {
      val as u32
    } else {
      core::u32::MAX
    }
  }
}
