use {
  crate::peer_list_manager::{PeerId, PeerListManager, PeerReputation},
  futures::Future,
  rand::{seq::SliceRandom, RngCore},
  std::{
    collections::{HashMap, HashSet},
    pin::Pin,
    task::{Context, Poll},
  },
};

pub struct SimplePeerListManager<R> {
  peers: HashMap<PeerId, PeerReputation>,
  rng: R,
}

impl<R> SimplePeerListManager<R> {
  pub fn build(rng: R) -> Self {
    SimplePeerListManager {
      peers: HashMap::new(),
      rng,
    }
  }
}

impl<R> Future for SimplePeerListManager<R> {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
    // Placeholder implementation
    Poll::Ready(())
  }
}

impl<R: RngCore> PeerListManager for SimplePeerListManager<R> {
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

  /// Returns a list of random peers
  fn get_random_peers(&mut self, n: usize) -> HashSet<PeerId> {
    let peer_ids: Vec<PeerId> = self.peers.keys().cloned().collect();
    if peer_ids.is_empty() {
      return Default::default();
    }

    // n is equal or greater than the number of peers, return all peers
    if peer_ids.len() <= n {
      return peer_ids.into_iter().collect();
    }

    // otherwise, select n random peers
    let mut selected_peers = std::collections::HashSet::new();

    while selected_peers.len() < n.min(peer_ids.len()) {
      if let Some(peer_id) = peer_ids.choose(&mut self.rng) {
        selected_peers.insert(peer_id.clone());
      }
    }

    selected_peers
  }
}
