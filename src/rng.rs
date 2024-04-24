use rand::{RngCore, SeedableRng};

pub struct RngSeed(pub u64);

pub trait GeneratesRngSeed<R> {
  fn next_rng_seed(&mut self) -> R;
}

impl<T: SeedableRng + RngCore> GeneratesRngSeed<T> for T {
  fn next_rng_seed(&mut self) -> T {
    Self::seed_from_u64(self.next_u64())
  }
}

