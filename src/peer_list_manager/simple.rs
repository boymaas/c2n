use {
  crate::peer_list_manager::{PeerId, PeerListManager, PeerReputation},
  futures::Future,
  rand::rngs::StdRng,
  std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll},
  },
};

pub struct SimplePeerListManager {
  peers: HashMap<PeerId, PeerReputation>,
  rng: StdRng,
}

impl SimplePeerListManager {
  pub fn build(rng: StdRng) -> Self {
    SimplePeerListManager {
      peers: HashMap::new(),
      rng,
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
