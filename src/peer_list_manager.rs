pub mod simple;

use {
  crate::types::{PeerId, PeerReputation},
  futures::Future,
};

pub trait PeerListManager: Future<Output = ()> {
  fn add_peer(&mut self, peer_id: PeerId, reputation: PeerReputation);
  fn remove_peer(&mut self, peer_id: &PeerId);
  fn get_peer_to_dial(&self) -> Option<PeerId>;
  fn update_peer_reputation(
    &mut self,
    peer_id: &PeerId,
    reputation_delta: PeerReputation,
  );
}
