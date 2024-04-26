use {
  super::{NetworkEvent, NetworkResult},
  crate::{
    network::{Network, NetworkError},
    types::{NodeAddress, PeerId},
  },
  futures::Future,
  rand::Rng,
  std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
  },
};

type RcClientMessageQueue = Rc<RefCell<VecDeque<Vec<u8>>>>;

// This is the simulation network responsible for forwarding the entwork
// messages to the network.
#[derive(Default)]
pub struct SimNetwork {
  clients: HashMap<PeerId, RcClientMessageQueue>,
}

impl SimNetwork {
  pub fn build() -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Default::default()))
  }

  pub fn connect(&mut self, peer_id: PeerId) -> Option<RcClientMessageQueue> {
    self.clients.get(&peer_id).map(Rc::clone)
  }

  pub fn register_client(
    &mut self,
    (peer_id, _): NodeAddress,
    queue: RcClientMessageQueue,
  ) {
    self.clients.insert(peer_id, queue);
  }
}

#[derive(Clone)]
pub struct SimNetworkClient<R> {
  rng: R,
  network: Rc<RefCell<SimNetwork>>,
  connections: HashMap<PeerId, RcClientMessageQueue>,
  queue: RcClientMessageQueue,
}

impl<R> Future for SimNetworkClient<R> {
  type Output = NetworkEvent;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating network...");
    Poll::Pending
  }
}

use crate::primitives::Pubkey;

impl<R: Rng> Network for SimNetworkClient<R> {
  fn send(&mut self, peer_id: PeerId, message: Vec<u8>) -> NetworkResult<()> {
    tracing::trace!(
      "Sending message: {} to peer_id {:?}",
      message.len(),
      peer_id
    );

    let connection = self
      .connections
      .get(&peer_id)
      .ok_or(NetworkError::NotConnected)?;

    connection.borrow_mut().push_back(message);

    Ok(())
  }

  fn add_peer(&mut self, peer_id: Pubkey, addr: NodeAddress) {
    tracing::debug!("Adding peer_id: {:?} with address: {:?}", peer_id, addr);
  }

  fn connect(&mut self, peer_id: PeerId) -> NetworkResult<()> {
    tracing::debug!("Connecting to peer_id: {:?}", peer_id);
    // TODO: connection is async and should happen when the connection
    // has been established which is based on some timeout.
    let client_queue = self
      .network
      .borrow_mut()
      .connect(peer_id)
      .ok_or(NetworkError::PeerNotFound)?;
    self.connections.insert(peer_id, client_queue);
    Ok(())
  }
}

impl<R: Rng> SimNetworkClient<R> {
  pub fn build(
    rng: R,
    network: Rc<RefCell<SimNetwork>>,
    address: NodeAddress,
  ) -> Self {
    let queue = Default::default();

    // register this client with the network so we can send messages
    network
      .borrow_mut()
      .register_client(address, Rc::clone(&queue));

    SimNetworkClient {
      rng,
      network,
      connections: Default::default(),
      queue,
    }
  }
}
