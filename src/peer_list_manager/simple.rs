use {
  super::{PeerListManagerConfig, PeerListManagerEvent},
  crate::peer_list_manager::{PeerId, PeerListManager, PeerReputation},
  futures::{Future, FutureExt},
  futures_timer::Delay,
  rand::{seq::SliceRandom, RngCore},
  std::{
    collections::{HashMap, HashSet},
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
  },
};

#[derive(Default, PartialEq, Eq)]
struct DialInfo {
  last_dial: Option<Instant>,
  attempts: u64,
}

#[derive(Default, PartialEq, Eq)]
enum PeerState {
  #[default]
  Disconnected,
  Connected,
  Dialing(DialInfo),
}

#[derive(Default)]
struct PeerInfo {
  reputation: PeerReputation,
  state: PeerState,
}

pub struct SimplePeerListManager<R> {
  config: PeerListManagerConfig,
  peers: HashMap<PeerId, PeerInfo>,
  exclude_peers: HashSet<PeerId>,
  interval: Delay,
  dial_interval: Delay,
  rng: R,
}

impl<R> SimplePeerListManager<R> {
  pub fn build(rng: R) -> Self {
    let config: PeerListManagerConfig = Default::default();
    SimplePeerListManager {
      interval: Delay::new(config.exchange_peers_interval),
      dial_interval: Delay::new(config.dial_interval),
      exclude_peers: Default::default(),
      config,
      peers: HashMap::new(),
      rng,
    }
  }

  pub fn connected_peers(&self) -> impl Iterator<Item = PeerId> + '_ {
    self.peers.iter().filter_map(|(peer_id, peer_info)| {
      if peer_info.state == PeerState::Connected {
        Some(*peer_id)
      } else {
        None
      }
    })
  }
}

impl<R: RngCore + Unpin> Future for SimplePeerListManager<R> {
  type Output = PeerListManagerEvent;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
    let this = self.get_mut();
    // check if the interval fired and select a random peer to request the
    // peer list from and return the sync event
    if let Poll::Ready(()) = this.interval.poll_unpin(_cx) {
      // reset the interval
      this.interval.reset(this.config.exchange_peers_interval);
      if let Some(peer_id) = this.get_random_peer() {
        return Poll::Ready(PeerListManagerEvent::SyncPeerList(peer_id));
      }
    }

    if let Poll::Ready(()) = this.dial_interval.poll_unpin(_cx) {
      // check if we have some peers to dial
      for (peer_id, peer_info) in this
        .peers
        .iter_mut()
        .filter(|(_, peer_info)| peer_info.state == PeerState::Disconnected)
      {
        peer_info.state = PeerState::Dialing(DialInfo::default());
        return Poll::Ready(PeerListManagerEvent::Dial(*peer_id));
      }
    }

    Poll::Pending
  }
}

impl<R: RngCore + Unpin> PeerListManager for SimplePeerListManager<R> {
  fn exclude_peer(&mut self, peer_id: PeerId) {
    self.exclude_peers.insert(peer_id);
  }

  fn register_peer(&mut self, peer_id: PeerId) {
    if self.exclude_peers.contains(&peer_id) {
      tracing::trace!("Peer {} is excluded from the peer list", peer_id);
      return;
    }
    self.peers.entry(peer_id).or_insert_with(PeerInfo::default);
  }

  fn remove_peer(&mut self, peer_id: &PeerId) {
    self.peers.remove(peer_id);
  }

  fn update_peer_reputation(
    &mut self,
    peer_id: &PeerId,
    reputation_delta: PeerReputation,
  ) {
    if let Some(peer_info) = self.peers.get_mut(peer_id) {
      peer_info.reputation += reputation_delta;
    }
  }

  /// Returna a single random peer
  fn get_random_peer(&mut self) -> Option<PeerId> {
    // TODO: optimize this
    let peer_ids: Vec<PeerId> = Vec::from_iter(self.connected_peers());
    if peer_ids.is_empty() {
      return None;
    }

    Some(peer_ids.choose(&mut self.rng).cloned().unwrap())
  }

  /// Returns a list of random peers
  fn get_random_peers(&mut self, n: usize) -> HashSet<PeerId> {
    // TODO: this can be optimized
    let peer_ids: Vec<PeerId> = Vec::from_iter(self.connected_peers());
    if peer_ids.is_empty() {
      return Default::default();
    }

    // n is equal or greater than the number of peers, return all peers
    if peer_ids.len() <= n {
      return peer_ids.into_iter().collect();
    }

    // otherwise, select n random peers
    let mut selected_peers = HashSet::new();

    while selected_peers.len() < n.min(peer_ids.len()) {
      if let Some(peer_id) = peer_ids.choose(&mut self.rng) {
        selected_peers.insert(peer_id.clone());
      }
    }

    selected_peers
  }

  fn register_peer_connected(&mut self, peer_id: PeerId) {
    // Check if the peer is already in the list and move it to the connected
    // list. If it is not registered, add it to the connected list.
    self.register_peer(peer_id);

    // Now make sure the stat of the peer is connected
    self.peers.get_mut(&peer_id).unwrap().state = PeerState::Connected;
  }

  fn register_peer_disconnected(&mut self, peer_id: PeerId) {
    // Check if this peer is in the list, and remove it from the connected list
    // if present.
    if let Some(peer) = self.peers.get_mut(&peer_id) {
      peer.state = PeerState::Disconnected;
    }
  }

  fn connections(&self) -> Vec<PeerId> {
    self.connected_peers().collect()
  }
}
