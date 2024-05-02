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
  Dial(PeerId),
}

/// Configuration for the PeerListManager
pub struct PeerListManagerConfig {
  pub max_peers: usize,
  /// The amount of peers exchanged on a peer list exchange
  pub exchange_peers: usize,
  /// The interval at which to exchange the peerlists
  pub exchange_peers_interval: Duration,
  /// The interval at which new peers will be dialed
  pub dial_interval: Duration,
}

impl Default for PeerListManagerConfig {
  fn default() -> Self {
    PeerListManagerConfig {
      max_peers: 1000,
      exchange_peers: 10,
      exchange_peers_interval: Duration::from_secs(2),
      dial_interval: Duration::from_secs(1),
    }
  }
}

/// The PeerListManager maintains connections between peer nodes in a network.
/// It registers new connections with `register_peer_connection`, and tracks
/// disconnections with `record_peer_disconnection`. The manager ensures the
/// network stays robust by initiating new connections if below a certain
/// threshold using `initiate_peer_dialing`.
///
/// Implementations should base their logic around the PeerListManagerConfig,
/// which can be used to design the behavior of the component.

pub trait PeerListManager: Future<Output = PeerListManagerEvent> {
  /// Called when a peer has been discovered. The PSM will determine if we are
  /// already connected or if we should connect.
  fn register_peer(&mut self, peer_id: PeerId);
  /// Called when a peer has been connected. Based on a Network Event
  fn register_peer_connected(&mut self, peer_id: PeerId);
  /// Called when a peer has been disconnected. Based on a Network Event
  fn register_peer_disconnected(&mut self, peer_id: PeerId);

  fn exclude_peer(&mut self, peer_id: PeerId);
  fn get_random_peer(&mut self) -> Option<PeerId>;

  /// returns a list of ranomd peers to which we are connected
  fn get_random_peers(&mut self, n: usize) -> HashSet<PeerId>;

  fn remove_peer(&mut self, peer_id: &PeerId);

  fn update_peer_reputation(
    &mut self,
    peer_id: &PeerId,
    reputation_delta: PeerReputation,
  );

  fn connections(&self) -> Vec<PeerId>;
}
