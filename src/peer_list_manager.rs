pub mod simple;

use {
  crate::types::{PeerId, PeerReputation},
  futures::Future,
  std::collections::HashSet,
};

/// Configuration for the PeerListManager
pub struct PeerListManagerConfig {
  pub max_peers: usize,
  /// The amount of peers exchanged on a peer list exchange
  pub exchange_peers: usize,
}

impl Default for PeerListManagerConfig {
  fn default() -> Self {
    PeerListManagerConfig {
      max_peers: 1000,
      exchange_peers: 10,
    }
  }
}

pub trait PeerListManager: Future<Output = ()> {
  fn add_peer(&mut self, peer_id: PeerId, reputation: PeerReputation);
  fn get_random_peers(&mut self, n: usize) -> HashSet<PeerId>;
  fn remove_peer(&mut self, peer_id: &PeerId);
  fn get_peer_to_dial(&self) -> Option<PeerId>;
  fn update_peer_reputation(
    &mut self,
    peer_id: &PeerId,
    reputation_delta: PeerReputation,
  );
}
