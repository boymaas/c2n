use {
  super::{NetworkEvent, NetworkResult},
  crate::{
    network::{Network, NetworkError},
    types::{NodeAddress, PeerId},
  },
  futures::{
    future::{BoxFuture, LocalBoxFuture},
    stream::FuturesUnordered,
    Future,
    FutureExt,
    StreamExt,
  },
  futures_timer::Delay,
  rand::Rng,
  std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    ops::Range,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
  },
};

type RcClientMessageQueue = Rc<RefCell<VecDeque<Vec<u8>>>>;

// Configuration for the simulation network.
pub struct SimNetworkConfig {
  connection_delay: Range<Duration>,
  connection_fail_prob: f64,
}

impl Default for SimNetworkConfig {
  fn default() -> Self {
    SimNetworkConfig {
      connection_delay: Duration::from_millis(100)..Duration::from_millis(500),
      connection_fail_prob: 0.1,
    }
  }
}

// Some outcomes
enum DialerOutcome {
  Success(PeerId),
  Failure(PeerId),
}

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

pub struct SimNetworkClient<R> {
  rng: R,
  config: SimNetworkConfig,
  network: Rc<RefCell<SimNetwork>>,
  connections: HashMap<PeerId, RcClientMessageQueue>,
  queue: RcClientMessageQueue,
  dialer: FuturesUnordered<LocalBoxFuture<'static, DialerOutcome>>,
}

impl<R: Unpin> Future for SimNetworkClient<R> {
  type Output = NetworkEvent;

  fn poll(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Self::Output> {
    let mut this = self.as_mut();

    // poll the dialer
    while let Poll::Ready(Some(outcome)) = this.dialer.poll_next_unpin(cx) {
      match outcome {
        DialerOutcome::Success(peer_id) => {
          tracing::debug!("DialerOutcome::Success: {}", peer_id);

          let maybe_client_queue = this.network.borrow_mut().connect(peer_id);
          if let Some(client_queue) = maybe_client_queue {
            this.connections.insert(peer_id, client_queue);
            return Poll::Ready(NetworkEvent::DialSucces { peer_id });
          }
          tracing::warn!(
            "DialerOutcome::Failure: {} Client is not connected",
            peer_id
          );
          return Poll::Ready(NetworkEvent::DialFailed { peer_id });
        }
        DialerOutcome::Failure(peer_id) => {
          tracing::debug!("DialerOutcome::Failure: {}", peer_id);
          return Poll::Ready(NetworkEvent::DialFailed { peer_id });
        }
      }
    }

    Poll::Pending
  }
}

use crate::primitives::Pubkey;

impl<R: Rng + Unpin> Network for SimNetworkClient<R> {
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
    tracing::debug!("Connecting to peer_id: {}", peer_id);
    // add to dialer with a random delay
    let delay = self.rng.gen_range(self.config.connection_delay.clone());
    let is_failure = self.rng.gen_bool(self.config.connection_fail_prob);
    let delayed_dialer_outcome = if is_failure {
      async move {
        Delay::new(delay).await;
        DialerOutcome::Failure(peer_id)
      }
      .boxed_local()
    } else {
      async move {
        Delay::new(delay).await;
        DialerOutcome::Success(peer_id)
      }
      .boxed_local()
    };

    self.dialer.push(delayed_dialer_outcome);

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
      // TODO: make this a var
      config: Default::default(),
      rng,
      network,
      connections: Default::default(),
      queue,
      dialer: Default::default(),
    }
  }
}
