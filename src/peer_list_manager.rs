pub mod simple;

use {
  futures::Future,
  std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll},
  },
};

pub type PeerId = String;
pub type PeerReputation = i32;

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
