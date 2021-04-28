use crate::Gen32;
use crate::*;


// Other multipliers: 0xffffffff0e703b65 0xf2fc5985
const PCG_MULTIPLIER_32: u32 = 0xf13283ad;

/// Advances a PCG with 64 bits of state.
macro_rules! pcg_core_state32 {
    ($state:expr, $inc:expr) => {
        $state.wrapping_mul(PCG_MULTIPLIER_32).wrapping_add($inc)
    };
}
macro_rules! rxs_m_xs_u32_to_u32 {
    ($state: expr) => {
        {
            $state ^= ($state >> (4 + ($state >> 28) as u32)) * 277803737u32;
            $state ^ ($state >> 22)
        }
    };
}
make_jump_lcgX!(jump_lcg32_32, u32);

/// A [permuted congruential
/// generator](https://en.wikipedia.org/wiki/Permuted_congruential_generator)
/// with 32 bits of output per step.
///
/// * Generally you should create new generator values with the
///   [`seed`](Self::seed) constructor. This will shuffle around the inputs
///   somewhat, so it will work alright even with "boring" input values like
///   `seed(0,0)` or whatever.
/// * If you want to exactly save/restore a generator use the `Into` and `From`
///   impls to convert the generator into and from a `[u32; 2]`.
/// * The methods on this type are quite minimal. You're expected to use the
///   [`Gen32`] trait to provide most of the useful operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pcg32x32 {
  state: u32,
  inc: u32,
}

impl Pcg32x32 {
  /// Seed a new generator.
  pub const fn seed(seed: u32, inc: u32) -> Self {
    let inc = (inc << 1) | 1;
    let mut state = pcg_core_state32!(0_u32, inc);
    state = state.wrapping_add(seed);
    state = pcg_core_state32!(state, inc);
    Self { state, inc }
  }

  /// Gets the next 32-bits of output.
  #[inline]
  pub fn next_u32(&mut self) -> u32 {
    // LLVM do the instruction-level parallelism plz ;_;
    let out = rxs_m_xs_u32_to_u32!(self.state);
    self.state = pcg_core_state32!(self.state, self.inc);
    out
  }

  /// Jumps the generator by `delta` steps forward.
  ///
  /// The generator sequence loops, so if you want to go "backwards" you can
  /// just subtract the number of steps you want to go back from `u32::MAX` and
  /// jump by that amount.
  #[inline]
  pub fn jump(&mut self, delta: u32) {
    self.state = jump_lcg32_32(delta, self.state, PCG_MULTIPLIER_32, self.inc);
  }
}

impl Default for Pcg32x32 {
  fn default() -> Self {
    const THE_DEFAULT: Pcg32x32 = Pcg32x32::seed(DEFAULT_PCG_SEED as _, DEFAULT_PCG_INC as _);
    THE_DEFAULT
  }
}

impl From<[u32; 2]> for Pcg32x32 {
  fn from([state, inc]: [u32; 2]) -> Self {
    Self { state, inc }
  }
}

impl From<Pcg32x32> for [u32; 2] {
  fn from(pcg: Pcg32x32) -> Self {
    [pcg.state, pcg.inc]
  }
}

impl Gen32 for Pcg32x32 {
  fn next_u32(&mut self) -> u32 {
    Pcg32x32::next_u32(self)
  }
}
