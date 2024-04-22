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

pub struct SimplePeerListManager {
  peers: HashMap<PeerId, PeerReputation>,
}

impl SimplePeerListManager {
  pub fn new() -> Self {
    SimplePeerListManager {
      peers: HashMap::new(),
    }
  }
}

impl Future for SimplePeerListManager {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
    // Placeholder implementation
    Poll::Ready(())
  }
}

impl PeerListManager for SimplePeerListManager {
  fn add_peer(&mut self, peer_id: PeerId, reputation: PeerReputation) {
    self.peers.insert(peer_id, reputation);
  }

  fn remove_peer(&mut self, peer_id: &PeerId) {
    self.peers.remove(peer_id);
  }

  fn get_peer_to_dial(&self) -> Option<PeerId> {
    // Placeholder implementation: return the first peer in the list
    self.peers.keys().next().cloned()
  }

  fn update_peer_reputation(
    &mut self,
    peer_id: &PeerId,
    reputation_delta: PeerReputation,
  ) {
    if let Some(reputation) = self.peers.get_mut(peer_id) {
      *reputation += reputation_delta;
    }
  }
}
