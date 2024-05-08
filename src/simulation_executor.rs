use {
  crate::{node::SimulatableNode, node_events::NodeEvent},
  futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    Future,
    FutureExt,
    StreamExt,
  },
  futures_timer::Delay,
  std::{pin::Pin, task::Context, time::Duration},
};

type SimulatableNodeFuture = Pin<Box<dyn SimulatableNode>>;

pub struct SimulationExecutor<N> {
  network: Pin<Box<N>>,
  delayed_join:
    FuturesUnordered<Pin<Box<dyn Future<Output = SimulatableNodeFuture>>>>,
  pub nodes: Vec<SimulatableNodeFuture>,
}

impl<N: Future<Output = ()>> SimulationExecutor<N> {
  pub fn new(network: Pin<Box<N>>) -> Self {
    SimulationExecutor {
      network,
      delayed_join: Default::default(),
      nodes: Vec::new(),
    }
  }

  pub fn add_node(&mut self, delay: Duration, node: SimulatableNodeFuture) {
    self
      .delayed_join
      .push(Delay::new(delay).map(move |_| node).boxed_local());
  }

  pub fn run_tick(&mut self) {
    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    // Attempt to progess the network
    if self.network.as_mut().poll(&mut cx).is_ready() {
      // The network has completed its operation
      // return;
    }

    // Attempt to progress any delayed nodes
    if let std::task::Poll::Ready(Some(node)) =
      self.delayed_join.poll_next_unpin(&mut cx)
    {
      self.nodes.push(node);
    }

    // Randomize the polling of the nodes and exit on the first exit event

    // Attempt to progress each node's state by one tick.
    for node in &mut self.nodes {
      let _ = node.as_mut().poll(&mut cx);
    }
  }

  // Here you would handle any logic to check if the simulation should continue
  // or if specific nodes have completed their operations.
}
