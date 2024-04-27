use {
  crate::{
    network::{Network, NetworkEvent, ProtocolMessage},
    node_config::{NodeConfig, NodeConfigBuilder},
    node_events::NodeEvent,
    peer_list_manager::PeerListManager,
    storage::Storage,
    types::PeerReputation,
  },
  futures::future::FutureExt,
  std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
  },
};

#[derive(Default)]
pub enum NodeState {
  #[default]
  Booting,
  Connecting,
  Joining,
  Running,
  Leaving,
  Stopped,
}

pub struct Node<N, S, P>
where
  N: Network,
  S: Storage,
  P: PeerListManager,
{
  config: NodeConfig,
  network: N,
  storage: S,
  peer_list_manager: P,

  state: NodeState,
}

impl<N, S, P> Node<N, S, P>
where
  N: Network,
  S: Storage,
  P: PeerListManager,
{
  pub fn builder() -> NodeBuilder<N, S, P> {
    NodeBuilder::new()
  }

  pub fn config_builder() -> NodeConfigBuilder {
    NodeConfigBuilder::new()
  }

  pub fn config(&self) -> &NodeConfig {
    &self.config
  }
}

impl<N, S, P> Node<N, S, P>
where
  N: Network + Unpin,
  S: Storage,
  P: PeerListManager,
{
  /// When the node is in the booting state, it will attempt to connect to the
  /// bootnode and activate the peer list manager to enable connections with
  /// the peers of the network
  fn poll_booting(&mut self, _cx: &mut Context<'_>) -> Poll<NodeEvent> {
    // let first make sure we connect to the bootnode
    // and discover enough other peers to try to join our consensus.
    for (peer_id, _) in self.config.bootnodes() {
      self
        .network
        .connect(*peer_id)
        .expect("Failed to connect to bootnode");
    }

    // move to the next state, waiting for dialing to succeed
    // and to connect to a certain amount of peers
    self.state = NodeState::Connecting;

    Poll::Ready(NodeEvent::Noop)
  }

  /// The node tries to connect to the bootnodes and tries to discover the
  /// network
  fn poll_connecting(&mut self, cx: &mut Context<'_>) -> Poll<NodeEvent> {
    // check if we have enough peers to join the network
    // if self.peer_list_manager.has_enough_peers() {
    //   // move to the next state
    //   self.state = NodeState::Joining;
    // }

    // handle the network event
    if let Poll::Ready(network_event) = self.network.poll_unpin(cx) {
      match network_event {
        NetworkEvent::IncomingEstablished { peer_id } => {
          tracing::debug!("IncomingEstablished: {:?}", peer_id);
          self
            .peer_list_manager
            .add_peer(peer_id, PeerReputation::default());

          // get a random list of peers to return
          let peers = self
            .peer_list_manager
            .get_random_peers(self.config.peer_list_manager.exchange_peers);

          self
            .network
            .send(peer_id, ProtocolMessage::PeerList { peers })
            .expect("Failed to send peerlist");

          return Poll::Ready(NodeEvent::IncomingEstablished { peer_id });
        }
        NetworkEvent::PeerDisconnected { peer_id } => {
          tracing::debug!("PeerDisconnected: {:?}", peer_id);
          return Poll::Ready(NodeEvent::PeerDisconnected { peer_id });
        }
        NetworkEvent::MessageReceived { peer_id, message } => {
          tracing::debug!("MessageReceived from {:?}: {:?}", peer_id, message);
          return Poll::Ready(NodeEvent::Noop);
        }
        NetworkEvent::DialSucces { peer_id } => {
          tracing::debug!("DialSucces: {}", peer_id);
          // add to the peer list manager
          self
            .peer_list_manager
            .add_peer(peer_id, PeerReputation::default());
        }
        NetworkEvent::DialFailed { peer_id } => {
          tracing::error!("DialFailed: {}", peer_id);
        }
      }
    }

    Poll::Pending
  }
}

impl<N, S, P> Future for Node<N, S, P>
where
  N: Network + Unpin,
  S: Storage + Unpin,
  P: PeerListManager + Unpin,
{
  type Output = NodeEvent;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();

    match this.state {
      NodeState::Booting => {
        return this.poll_booting(cx);
      }
      NodeState::Connecting => {
        return this.poll_connecting(cx);
      }
      NodeState::Joining => {}
      NodeState::Running => {
        // The node is running
      }
      NodeState::Leaving => {
        // The node is leaving
      }
      NodeState::Stopped => {
        // The node is stopped
      }
    }

    // handle the storage events
    if let Poll::Ready(()) = this.storage.poll_unpin(cx) {
      // Storage has completed its work
    }

    // This example does not complete, to illustrate ongoing processing.
    // Adjust according to your simulation's end conditions.
    Poll::Pending
  }
}

// Builder pattern for Node
pub struct NodeBuilder<N, S, P> {
  config: Option<NodeConfig>,
  network: Option<N>,
  storage: Option<S>,
  peer_list_manager: Option<P>,
}

impl<N, S, P> Default for NodeBuilder<N, S, P>
where
  N: Network,
  S: Storage,
  P: PeerListManager,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<N, S, P> NodeBuilder<N, S, P>
where
  N: Network,
  S: Storage,
  P: PeerListManager,
{
  pub fn new() -> Self {
    Self {
      network: None,
      storage: None,
      peer_list_manager: None,
      config: None,
    }
  }

  pub fn network(mut self, network: N) -> Self {
    self.network = Some(network);
    self
  }

  pub fn storage(mut self, storage: S) -> Self {
    self.storage = Some(storage);
    self
  }

  pub fn with_node_config(mut self, config: NodeConfig) -> Self {
    // Apply the configuration to the node.
    // This could be used to set up the node's behavior based on the
    // configuration.
    self.config = Some(config);
    self
  }

  pub fn peer_list_manager(mut self, peer_list_manager: P) -> Self {
    self.peer_list_manager = Some(peer_list_manager);
    self
  }

  pub fn build(self) -> Node<N, S, P> {
    Node {
      state: Default::default(),
      config: self.config.expect("Node configuration is required"),
      network: self.network.expect("Network component is required"),
      storage: self.storage.expect("Storage component is required"),
      peer_list_manager: self
        .peer_list_manager
        .expect("Peer list manager is required"),
    }
  }
}
