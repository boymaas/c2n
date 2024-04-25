use {
  crate::{
    network::{Network, NetworkEvent},
    node_config::{NodeConfig, NodeConfigBuilder},
    node_events::NodeEvent,
    peer_list_manager::PeerListManager,
    storage::Storage,
  },
  futures::{future::FutureExt, ready},
  std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
  },
};

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

impl<N, S, P> Future for Node<N, S, P>
where
  N: Network + Unpin,
  S: Storage + Unpin,
  P: PeerListManager + Unpin,
{
  type Output = NodeEvent;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();

    // handle the network event
    if let Poll::Ready(network_event) = this.network.poll_unpin(cx) {
      match network_event {
        NetworkEvent::PeerConnected { peer_id } => {
          tracing::debug!("PeerConnected: {:?}", peer_id);
          return Poll::Ready(NodeEvent::PeerConnected { peer_id });
        }
        NetworkEvent::PeerDisconnected { peer_id } => {
          tracing::debug!("PeerDisconnected: {:?}", peer_id);
          return Poll::Ready(NodeEvent::PeerDisconnected { peer_id });
        }
        NetworkEvent::MessageReceived { peer_id, message } => {
          tracing::debug!("MessageReceived from {:?}: {:?}", peer_id, message);
          return Poll::Ready(NodeEvent::Noop);
        }
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
      config: self.config.expect("Node configuration is required"),
      network: self.network.expect("Network component is required"),
      storage: self.storage.expect("Storage component is required"),
      peer_list_manager: self
        .peer_list_manager
        .expect("Peer list manager is required"),
    }
  }
}
