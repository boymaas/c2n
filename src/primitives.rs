use rand::Rng;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Pubkey {
  key: [u8; 32],
}

impl Pubkey {
  // generate a new random key
  pub fn unique(rng: &mut impl Rng) -> Self {
    // generate a random key
    Pubkey { key: rng.gen() }
  }
}
