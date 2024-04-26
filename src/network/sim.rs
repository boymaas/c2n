use {
  super::NetworkEvent,
  crate::{
    network::Network,
    types::{NodeAddress, PeerId},
  },
  futures::Future,
  rand::{rngs::StdRng, Rng},
  std::{
    pin::Pin,
    task::{Context, Poll},
  },
};

#[derive(Clone)]
pub struct SimNetwork<R> {
  rng: R,
}

impl<R> Future for SimNetwork<R> {
  type Output = NetworkEvent;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating network...");
    Poll::Pending
  }
}

use crate::primitives::Pubkey;

impl<R: Rng> Network for SimNetwork<R> {
  fn send(&mut self, peer_id: PeerId, message: Vec<u8>) {
    tracing::debug!(
      "Sending message: {} to peer_id {:?}",
      message.len(),
      peer_id
    );
  }

  fn add_peer(&mut self, peer_id: Pubkey, addr: NodeAddress) {
    tracing::debug!("Adding peer_id: {:?} with address: {:?}", peer_id, addr);
  }

  fn connect(&mut self, peer_id: PeerId) {
    tracing::debug!("Connecting to peer_id: {:?}", peer_id);
  }
}

impl<R: Rng> SimNetwork<R> {
  pub fn build(rng: R) -> Self {
    SimNetwork { rng }
  }
}
