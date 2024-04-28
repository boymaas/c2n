pub mod memory;
pub mod sim;

use {
  crate::{
    primitives::Pubkey,
    types::{NodeAddress, PeerId},
  },
  std::{collections::HashSet, future::Future},
  thiserror::Error,
};

#[derive(Debug, Error)]
pub enum NetworkError {
  #[error("already connected")]
  AlreadyConnected(PeerId),
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
  fn send(
    &mut self,
    peer_id: PeerId,
    message: ProtocolMessage,
  ) -> NetworkResult<()>;
}

/// Protocol Messages that can be send over the network
#[derive(Debug)]
pub enum ProtocolMessage {
  /// A random PeerList communicating a set of peers I am connected to.
  PeerList { peers: HashSet<PeerId> },
  /// A message that is sent to a peer to exchange data.
  Data { data: Vec<u8> },
}

/// Events that can be emitted by a network.
#[derive(Debug)]
pub enum NetworkEvent {
  /// Indicates that a dial attempt has succeeded.
  DialSucces { peer_id: Pubkey },
  /// A dial attempt has failed.
  DialFailed { peer_id: Pubkey },
  /// A new peer has connected to the network.
  IncomingEstablished { peer_id: Pubkey },
  /// A peer has disconnected from the network.
  PeerDisconnected { peer_id: Pubkey },
  /// A message has been received from a peer.
  MessageReceived {
    peer_id: Pubkey,
    message: ProtocolMessage,
  },
}
