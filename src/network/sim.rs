use {
  super::NetworkEvent,
  crate::{network::Network, types::PeerId},
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
  fn send(&mut self, peer_id: PeerId, message: String) {
    println!("Sending message: {} to peer_id {:?}", message, peer_id);
  }
}

impl<R: Rng> SimNetwork<R> {
  pub fn build(rng: R) -> Self {
    SimNetwork { rng }
  }
}
