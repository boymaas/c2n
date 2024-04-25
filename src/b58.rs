use crate::primitives::Pubkey;

pub trait Base58Encode {
  fn bs58_encode(&self) -> String;
}

impl Base58Encode for [u8] {
  fn bs58_encode(&self) -> String {
    bs58::encode(self).into_string()
  }
}

impl Base58Encode for Pubkey {
  fn bs58_encode(&self) -> String {
    self.to_bytes().bs58_encode()
  }
}
