use {
  crate::{b58::Base58Encode, types::NodeAddress},
  core::fmt,
  multiaddr::Multiaddr,
  rand::Rng,
  std::fmt::{Display, Formatter},
};

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

  pub fn to_bytes(&self) -> [u8; 32] {
    self.key
  }

  pub fn into_node_address(&self, addr: Multiaddr) -> NodeAddress {
    (*self, addr)
  }
}

impl std::fmt::Debug for Pubkey {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.bs58_encode())
  }
}

impl Display for Pubkey {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.bs58_encode())
  }
}
