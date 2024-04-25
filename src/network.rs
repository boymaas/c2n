pub mod memory;
pub mod sim;

use {
  crate::{
    primitives::Pubkey,
    types::{NodeAddress, PeerId},
  },
  std::future::Future,
};

/// A network interface that can send and receive messages and emit network
/// events.
pub trait Network: Future<Output = NetworkEvent> {
  fn add_peer(&mut self, peer_id: Pubkey, addr: NodeAddress);
  fn connect(&mut self, peer_id: PeerId);
  fn send(&mut self, peer_id: Pubkey, message: String);
}

pub enum NetworkEvent {
  /// A new peer has connected to the network.
  PeerConnected { peer_id: Pubkey },
  /// A peer has disconnected from the network.
  PeerDisconnected { peer_id: Pubkey },
  /// A message has been received from a peer.
  MessageReceived { peer_id: Pubkey, message: Vec<u8> },
}
