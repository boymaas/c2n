use {
  super::{NetworkEvent, NetworkResult, ProtocolMessage},
  crate::{
    network::{Network, NetworkError},
    primitives::Pubkey,
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

// TODO: The protocol message should include a from PeerId to indicate the
// origin of the message. Once this is added, we will be able to track the
// communication between peers.
type RcProtocolMessageQueue = Rc<RefCell<VecDeque<(PeerId, ProtocolMessage)>>>;
type RcSimNetworkEventQueue = Rc<RefCell<VecDeque<(PeerId, SimNetworkEvent)>>>;

// Configuration for the simulation network.
#[derive(Clone)]
pub struct SimNetworkConfig {
  connection_delay: Range<Duration>,
  connection_fail_prob: f64,
}

impl Default for SimNetworkConfig {
  fn default() -> Self {
    SimNetworkConfig {
      connection_delay: Duration::from_millis(100)..Duration::from_millis(2000),
      connection_fail_prob: 0.1,
    }
  }
}

pub enum SimNetworkEvent {
  InboundEstablished {
    from: PeerId,
    queue: RcProtocolMessageQueue,
  },
  InboundFailure {
    from: PeerId,
  },
  OutboundEstablished {
    to: PeerId,
    queue: RcProtocolMessageQueue,
  },
  OutboundFailure {
    to: PeerId,
  },
}

// Some outcomes
enum DialerOutcome {
  Success(PeerId, PeerId),
  Failure(PeerId, PeerId),
}

#[derive(Clone)]
pub struct ClientConnection {
  peer_id: PeerId,
  queue: RcProtocolMessageQueue,
  events: RcSimNetworkEventQueue,
  config: SimNetworkConfig,
}

impl ClientConnection {
  pub fn send_message(&self, from: PeerId, message: ProtocolMessage) {
    self.queue.borrow_mut().push_back((from, message));
  }

  pub fn push_event(&self, from: PeerId, event: SimNetworkEvent) {
    self.events.borrow_mut().push_back((from, event));
  }
}

// This is the simulation network responsible for forwarding the entwork
// messages to the network.
pub struct SimNetwork<R> {
  rng: R,
  clients: HashMap<PeerId, ClientConnection>,
  dialer: FuturesUnordered<LocalBoxFuture<'static, DialerOutcome>>,
}

pub struct SimNetworkFuture<R>(pub Rc<RefCell<SimNetwork<R>>>);

impl<R> SimNetworkFuture<R> {
  pub fn wrap(network: &Rc<RefCell<SimNetwork<R>>>) -> Self {
    Self(Rc::clone(network))
  }
}

impl<R: Unpin> Future for SimNetworkFuture<R> {
  type Output = ();

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    this.0.borrow_mut().poll_unpin(cx)
  }
}

impl<R: Unpin> Future for SimNetwork<R> {
  type Output = ();

  fn poll(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Self::Output> {
    let mut this = self.as_mut();

    while let Poll::Ready(Some(outcome)) = this.dialer.poll_next_unpin(cx) {
      match outcome {
        DialerOutcome::Success(from_peer_id, to_peer_id) => {
          // create a connection
          let from_connection = this.clients.get(&from_peer_id).unwrap();
          let to_connection = this.clients.get(&to_peer_id).unwrap();
          from_connection.push_event(
            to_peer_id,
            SimNetworkEvent::OutboundEstablished {
              to: to_peer_id,
              queue: Rc::clone(&to_connection.queue),
            },
          );

          tracing::warn!(
            "InboundEstablished from: {:?} to: {:?}",
            from_peer_id,
            to_peer_id,
          );
          let from_connection = this.clients.get(&from_peer_id).unwrap();
          let to_connection = this.clients.get(&to_peer_id).unwrap();
          to_connection.push_event(
            from_peer_id,
            SimNetworkEvent::InboundEstablished {
              from: from_peer_id,
              queue: Rc::clone(&from_connection.queue),
            },
          );
        }
        DialerOutcome::Failure(from_peer_id, to_peer_id) => {
          let from_connection = this.clients.get(&from_peer_id).unwrap();
          from_connection
            .push_event(to_peer_id, SimNetworkEvent::OutboundFailure {
              to: to_peer_id,
            });

          let to_connection = this.clients.get(&to_peer_id).unwrap();
          to_connection
            .push_event(from_peer_id, SimNetworkEvent::InboundFailure {
              from: from_peer_id,
            });
        }
      }
    }

    Poll::Pending
  }
}

impl<R> SimNetwork<R> {
  pub fn build(rng: R) -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self {
      rng,
      clients: Default::default(),
      dialer: Default::default(),
    }))
  }

  pub fn register_client(&mut self, client: &SimNetworkClient<R>) {
    self.clients.insert(client.peer_id(), client.connection());
  }
}

impl<R: Rng> SimNetwork<R> {
  pub fn connect(&mut self, from_peer_id: PeerId, to_peer_id: PeerId) {
    // here we create a dialer entry
    // add to dialer with a random delay
    let from = self.clients.get(&from_peer_id).unwrap();
    // let to = self.clients.get(&to_peer_id).unwrap();

    let delay = self.rng.gen_range(from.config.connection_delay.clone());
    let is_failure = self.rng.gen_bool(from.config.connection_fail_prob);
    let delayed_dialer_outcome = if is_failure {
      async move {
        Delay::new(delay).await;
        DialerOutcome::Failure(from_peer_id, to_peer_id)
      }
      .boxed_local()
    } else {
      async move {
        Delay::new(delay).await;
        DialerOutcome::Success(from_peer_id, to_peer_id)
      }
      .boxed_local()
    };

    self.dialer.push(delayed_dialer_outcome);
  }
}

pub struct SimNetworkClient<R> {
  rng: R,
  config: SimNetworkConfig,
  address: NodeAddress,
  network: Rc<RefCell<SimNetwork<R>>>,
  connections: HashMap<PeerId, RcProtocolMessageQueue>,
  queue: RcProtocolMessageQueue,
  events: RcSimNetworkEventQueue,
}

impl<R: Unpin> Future for SimNetworkClient<R> {
  type Output = NetworkEvent;

  fn poll(
    mut self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
  ) -> Poll<Self::Output> {
    let mut this = self.as_mut();
    // check for simnetwork events
    let maybe_sim_network_event = this.events.borrow_mut().pop_front();
    if let Some((from_peer_id, event)) = maybe_sim_network_event {
      match event {
        SimNetworkEvent::InboundEstablished { from, queue } => {
          this.connections.insert(from, queue);
          return Poll::Ready(NetworkEvent::InboundEstablished {
            peer_id: from,
          });
        }
        SimNetworkEvent::InboundFailure { from } => {
          return Poll::Ready(NetworkEvent::OutboundFailure { peer_id: from });
        }
        SimNetworkEvent::OutboundEstablished { to, queue } => {
          this.connections.insert(to, queue);
          return Poll::Ready(NetworkEvent::OutboundEstablished {
            peer_id: to,
          });
        }
        SimNetworkEvent::OutboundFailure { to } => {
          return Poll::Ready(NetworkEvent::OutboundFailure { peer_id: to });
        }
      }
    }

    // if we have protocol message in our queue, return it as a network event
    if let Some((from_peer_id, message)) = this.queue.borrow_mut().pop_front() {
      return Poll::Ready(NetworkEvent::MessageReceived {
        peer_id: from_peer_id,
        message,
      });
    }

    Poll::Pending
  }
}

impl<R: Rng + Unpin> Network for SimNetworkClient<R> {
  fn send(
    &mut self,
    peer_id: PeerId,
    message: ProtocolMessage,
  ) -> NetworkResult<()> {
    let connection = self
      .connections
      .get(&peer_id)
      .ok_or(NetworkError::NotConnected)?;

    connection.borrow_mut().push_back((self.peer_id(), message));

    Ok(())
  }

  fn add_peer(&mut self, peer_id: Pubkey, addr: NodeAddress) {
    tracing::debug!("Adding peer_id: {:?} with address: {:?}", peer_id, addr);
  }

  fn connect(&mut self, peer_id: PeerId) -> NetworkResult<()> {
    tracing::debug!("Connect from {} peer_id: {}", self.peer_id(), peer_id);
    if self.connections.contains_key(&peer_id) {
      return Err(NetworkError::AlreadyConnected(peer_id));
    }

    // trigger the simulation network to connect
    self.network.borrow_mut().connect(self.peer_id(), peer_id);

    Ok(())
  }
}

impl<R> SimNetworkClient<R> {
  pub fn peer_id(&self) -> PeerId {
    self.address.0
  }

  fn connection(&self) -> ClientConnection {
    ClientConnection {
      peer_id: self.peer_id(),
      queue: Rc::clone(&self.queue),
      events: Rc::clone(&self.events),
      config: self.config.clone(),
    }
  }
}

impl<R: Rng> SimNetworkClient<R> {
  pub fn build(
    rng: R,
    network: Rc<RefCell<SimNetwork<R>>>,
    address: NodeAddress,
  ) -> Self {
    let queue = Default::default();
    let events = Default::default();

    let client = SimNetworkClient {
      // TODO: make this a var
      config: Default::default(),
      rng,
      address,
      network: Rc::clone(&network),
      connections: Default::default(),
      queue,
      events,
    };

    // register this client with the network so we can send messages
    network.borrow_mut().register_client(&client);

    client
  }
}
