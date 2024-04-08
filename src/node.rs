use {
  crate::{
    network::Network,
    node_config::{NodeConfig, NodeConfigBuilder},
    storage::Storage,
  },
  futures::ready,
  std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
  },
};

pub struct Noop;
impl Future for Noop {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Ready(())
  }
}

impl Network for Noop {
  fn send(&mut self, _message: String) {}

  fn receive(&mut self) -> Vec<String> {
    Vec::new()
  }
}

impl Storage for Noop {
  fn write(&mut self, _data: String) {}

  fn read(&mut self) -> String {
    String::new()
  }
}

pub struct Node<N = Noop, S = Noop>
where
  N: Network,
  S: Storage,
{
  config: NodeConfig,
  network: N,
  storage: S,
}

impl<N, S> Node<N, S>
where
  N: Network,
  S: Storage,
{
  pub fn builder() -> NodeBuilder<N, S> {
    NodeBuilder::new()
  }

  pub fn config_builder() -> NodeConfigBuilder {
    NodeConfigBuilder::new()
  }

  pub fn config(&self) -> &NodeConfig {
    &self.config
  }
}

impl<N, S> Future for Node<N, S>
where
  N: Network + Unpin,
  S: Storage + Unpin,
{
  type Output = ();

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    // Example of progressing network and storage components.
    // Here, you'd likely manage state transitions or operations
    // that need to happen over multiple ticks.

    // For demonstration, assume we simply call poll on both
    // components to simulate progressing their states.
    ready!(Pin::new(&mut this.network).poll(cx));
    ready!(Pin::new(&mut this.storage).poll(cx));

    // This example does not complete, to illustrate ongoing processing.
    // Adjust according to your simulation's end conditions.
    Poll::Pending
  }
}

// Builder pattern for Node
pub struct NodeBuilder<N, S> {
  config: Option<NodeConfig>,
  network: Option<N>,
  storage: Option<S>,
}

impl<N, S> NodeBuilder<N, S>
where
  N: Network,
  S: Storage,
{
  pub fn new() -> Self {
    Self {
      network: None,
      storage: None,
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

  pub fn build(self) -> Node<N, S> {
    Node {
      config: self.config.expect("Node configuration is required"),
      network: self.network.expect("Network component is required"),
      storage: self.storage.expect("Storage component is required"),
    }
  }
}
