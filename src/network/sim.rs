use {
  super::{NetworkEvent, NetworkResult, ProtocolMessage},
  crate::{
    network::{Network, NetworkError},
    types::{NodeAddress, PeerId},
  },
  futures::{
    future::LocalBoxFuture,
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

type RcProtocolMessageQueue = Rc<RefCell<VecDeque<ProtocolMessage>>>;
type RcNetworkEventQueue = Rc<RefCell<VecDeque<NetworkEvent>>>;

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

#[derive(Default, Clone)]
pub struct ClientConnection {
  queue: RcProtocolMessageQueue,
  events: RcNetworkEventQueue,
}

impl ClientConnection {
  pub fn send_message(&mut self, message: ProtocolMessage) {
    self.queue.borrow_mut().push_back(message);
  }

  pub fn push_event(&mut self, event: NetworkEvent) {
    self.events.borrow_mut().push_back(event);
  }
}

// This is the simulation network responsible for forwarding the entwork
// messages to the network.
#[derive(Default)]
pub struct SimNetwork {
  clients: HashMap<PeerId, ClientConnection>,
}

impl SimNetwork {
  pub fn build() -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Default::default()))
  }

  pub fn connect(&mut self, peer_id: PeerId) -> Option<ClientConnection> {
    self.clients.get(&peer_id).cloned()
  }

  pub fn register_client(
    &mut self,
    (peer_id, _): NodeAddress,
    queue: RcProtocolMessageQueue,
    events: RcNetworkEventQueue,
  ) {
    self
      .clients
      .insert(peer_id, ClientConnection { queue, events });
  }
}

pub struct SimNetworkClient<R> {
  rng: R,
  config: SimNetworkConfig,
  address: NodeAddress,
  network: Rc<RefCell<SimNetwork>>,
  connections: HashMap<PeerId, RcProtocolMessageQueue>,
  queue: RcProtocolMessageQueue,
  events: RcNetworkEventQueue,
  dialer: FuturesUnordered<LocalBoxFuture<'static, DialerOutcome>>,
}

impl<R: Unpin> Future for SimNetworkClient<R> {
  type Output = NetworkEvent;

  fn poll(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Self::Output> {
    let mut this = self.as_mut();

    // if we have an event in our queue, return it as an network event
    if let Some(event) = this.events.borrow_mut().pop_front() {
      tracing::debug!("NetworkEvent: {:?}", event);
      return Poll::Ready(event);
    }

    // if we have protocol message in our queue, return it as a network event
    if let Some(message) = this.queue.borrow_mut().pop_front() {
      tracing::debug!("MessageReceived from: {:?}", message);
      return Poll::Ready(NetworkEvent::MessageReceived {
        peer_id: this.address.0.clone(),
        message,
      });
    }

    // poll the dialer
    while let Poll::Ready(Some(outcome)) = this.dialer.poll_next_unpin(cx) {
      match outcome {
        DialerOutcome::Success(peer_id) => {
          tracing::debug!("DialerOutcome::Success: {}", peer_id);

          let maybe_client_connection =
            this.network.borrow_mut().connect(peer_id);
          if let Some(mut client_connection) = maybe_client_connection {
            // Generate the incoming established event on the
            // receiving end.
            client_connection.push_event(NetworkEvent::IncomingEstablished {
              peer_id: this.peer_id(),
            });
            // Register the connection with the client for only the protocol
            // messages we can send
            this.connections.insert(peer_id, client_connection.queue);
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
  fn send(
    &mut self,
    peer_id: PeerId,
    message: ProtocolMessage,
  ) -> NetworkResult<()> {
    tracing::trace!("Sending message: {:?} to peer_id {:?}", message, peer_id);

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

impl<R> SimNetworkClient<R> {
  pub fn peer_id(&self) -> PeerId {
    self.address.0.clone()
  }
}

impl<R: Rng> SimNetworkClient<R> {
  pub fn build(
    rng: R,
    network: Rc<RefCell<SimNetwork>>,
    address: NodeAddress,
  ) -> Self {
    let queue = Default::default();
    let events = Default::default();

    // register this client with the network so we can send messages
    network.borrow_mut().register_client(
      address.clone(),
      Rc::clone(&queue),
      Rc::clone(&events),
    );

    SimNetworkClient {
      // TODO: make this a var
      config: Default::default(),
      rng,
      address,
      network,
      connections: Default::default(),
      queue,
      dialer: Default::default(),
      events,
    }
  }
}
