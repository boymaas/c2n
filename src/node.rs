use {
  crate::{
    network::{Network, NetworkEvent, ProtocolMessage},
    node_config::{NodeConfig, NodeConfigBuilder},
    node_events::NodeEvent,
    peer_list_manager::{PeerListManager, PeerListManagerEvent},
    storage::Storage,
    types::PeerId,
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

  pub fn connections(&self) -> Vec<PeerId> {
    self.peer_list_manager.connections()
  }

  pub fn identity(&self) -> &PeerId {
    self.config.identity()
  }
}

impl<N, S, P> Node<N, S, P>
where
  N: Network + Unpin,
  S: Storage,
  P: PeerListManager + Unpin,
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
  #[tracing::instrument(skip(self, cx), fields(peer_id=%self.config.identity()))]
  fn poll_connecting(&mut self, cx: &mut Context<'_>) -> Poll<NodeEvent> {
    // check if we have enough peers to join the network
    // if self.peer_list_manager.has_enough_peers() {
    //   // move to the next state
    //   self.state = NodeState::Joining;
    // }

    // check if the peerlist manager has anything to do
    if let Poll::Ready(peer_list_manager_event) =
      self.peer_list_manager.poll_unpin(cx)
    {
      match peer_list_manager_event {
        PeerListManagerEvent::SyncPeerList(peer_id) => {
          let peers = self
            .peer_list_manager
            .get_random_peers(self.config.peer_list_manager.exchange_peers);
          self
            .network
            .send(peer_id, ProtocolMessage::PeerList { peers })
            .expect("Failed to send peerlist");
        }
        PeerListManagerEvent::PeerAdded(_, _) => {}
        PeerListManagerEvent::PeerRemoved(_) => {}
        PeerListManagerEvent::PeerReputationUpdated(_, _) => {}
        PeerListManagerEvent::Diconnect(peer_id) => {
          self.network.disconnect(peer_id);
        }
        PeerListManagerEvent::Dial(peer_id) => {
          self.network.connect(peer_id).expect("Failed to dial peer");
        }
      }
    }

    // handle the network event
    if let Poll::Ready(network_event) = self.network.poll_unpin(cx) {
      match network_event {
        NetworkEvent::InboundEstablished { peer_id } => {
          tracing::debug!("InboundEstablished: {:?}", peer_id);
          self.peer_list_manager.register_peer_connected(peer_id);

          // get a random list of peers to return
          let peers = self
            .peer_list_manager
            .get_random_peers(self.config.peer_list_manager.exchange_peers);

          self
            .network
            .send(peer_id, ProtocolMessage::PeerList { peers })
            .expect("Failed to send peerlist");

          return Poll::Ready(NodeEvent::InboundEstablished { peer_id });
        }
        NetworkEvent::PeerDisconnected { peer_id } => {
          tracing::debug!("PeerDisconnected: {:?}", peer_id);
          // remove from peer_list_manager
          self.peer_list_manager.register_peer_disconnected(peer_id);
          return Poll::Ready(NodeEvent::PeerDisconnected { peer_id });
        }
        NetworkEvent::MessageReceived { peer_id, message } => {
          tracing::debug!("MessageReceived from {:?}: {:?}", peer_id, message);
          match message {
            ProtocolMessage::PeerList { peers } => {
              for peer_id in peers {
                self.peer_list_manager.register_peer(peer_id);
              }
            }
          }
          return Poll::Ready(NodeEvent::Noop);
        }
        NetworkEvent::OutboundEstablished { peer_id } => {
          tracing::debug!("OutboundEstablished: {}", peer_id);
          // add to the peer list manager
          self.peer_list_manager.register_peer_connected(peer_id);
        }
        NetworkEvent::OutboundFailure { peer_id } => {
          tracing::error!("OutboundFailed: {}", peer_id);
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
    let config = self.config.expect("Node configuration is required");
    let mut peer_list_manager = self
      .peer_list_manager
      .expect("Peer list manager is required");

    // exclude our ientity from the peer list manager
    peer_list_manager.exclude_peer(*config.identity());

    Node {
      state: Default::default(),
      config,
      network: self.network.expect("Network component is required"),
      storage: self.storage.expect("Storage component is required"),
      peer_list_manager,
    }
  }
}
