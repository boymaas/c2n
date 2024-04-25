use {
  crate::{
    network::{Network, NetworkEvent},
    primitives::Pubkey,
    types::PeerId,
  },
  futures::Future,
  std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
  },
};

pub struct MemoryNetwork {
  events: VecDeque<NetworkEvent>,
}

impl MemoryNetwork {
  pub fn new() -> Self {
    MemoryNetwork {
      events: VecDeque::new(),
    }
  }
}

impl Future for MemoryNetwork {
  type Output = NetworkEvent;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Pending
  }
}

impl Network for MemoryNetwork {
  fn send(&mut self, _peer_id: PeerId, _message: String) {
    // Simulate sending a message
  }
}
