use rand::Rng;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Pubkey {
  key: [u8; 32],
}

impl Pubkey {
  // generate a new random key
  pub fn unique<R: Rng>(rng: &mut R) -> Self {
    // generate a random key
    Pubkey { key: rng.gen() }
  }
}
