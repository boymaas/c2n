pub mod memory;
pub mod sim;

use {
  crate::{
    primitives::Pubkey,
    types::{NodeAddress, PeerId},
  },
  std::future::Future,
  thiserror::Error,
};

#[derive(Debug, Error)]
pub enum NetworkError {
  #[error("peer not found")]
  PeerNotFound,
  #[error("not connected")]
  NotConnected,
}

pub type NetworkResult<T> = Result<T, NetworkError>;

/// A network interface that can send and receive messages and emit network
/// events.
pub trait Network: Future<Output = NetworkEvent> {
  fn add_peer(&mut self, peer_id: Pubkey, addr: NodeAddress);
  fn connect(&mut self, peer_id: PeerId) -> NetworkResult<()>;
  fn send(&mut self, peer_id: PeerId, message: Vec<u8>) -> NetworkResult<()>;
}

pub enum NetworkEvent {
  /// Indicates that a dial attempt has succeeded.
  DialSucces { peer_id: Pubkey },
  /// A dial attempt has failed.
  DialFailed { peer_id: Pubkey },
  /// A new peer has connected to the network.
  PeerConnected { peer_id: Pubkey },
  /// A peer has disconnected from the network.
  PeerDisconnected { peer_id: Pubkey },
  /// A message has been received from a peer.
  MessageReceived { peer_id: Pubkey, message: Vec<u8> },
}
