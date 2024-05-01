pub mod simple;

use {
  crate::types::{PeerId, PeerReputation},
  futures::Future,
  std::{collections::HashSet, time::Duration},
};

pub enum PeerListManagerEvent {
  PeerAdded(PeerId, PeerReputation),
  PeerRemoved(PeerId),
  PeerReputationUpdated(PeerId, PeerReputation),
  SyncPeerList(PeerId),
}

/// Configuration for the PeerListManager
pub struct PeerListManagerConfig {
  pub max_peers: usize,
  /// The amount of peers exchanged on a peer list exchange
  pub exchange_peers: usize,
  /// The interval at which to exchange the peerlists
  pub exchange_peers_interval: Duration,
}

impl Default for PeerListManagerConfig {
  fn default() -> Self {
    PeerListManagerConfig {
      max_peers: 1000,
      exchange_peers: 10,
      exchange_peers_interval: Duration::from_secs(2),
    }
  }
}

pub trait PeerListManager: Future<Output = PeerListManagerEvent> {
  fn add_peer(&mut self, peer_id: PeerId, reputation: PeerReputation);
  fn exclude_peer(&mut self, peer_id: PeerId);
  fn get_random_peer(&mut self) -> Option<PeerId>;
  fn get_random_peers(&mut self, n: usize) -> HashSet<PeerId>;
  fn remove_peer(&mut self, peer_id: &PeerId);
  fn update_peer_reputation(
    &mut self,
    peer_id: &PeerId,
    reputation_delta: PeerReputation,
  );
}
