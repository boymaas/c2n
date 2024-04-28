use {
  crate::node_events::NodeEvent,
  futures::{task::Poll, Future},
  std::{pin::Pin, task::Context},
};

pub struct SimulationExecutor<N, Node> {
  network: Pin<Box<N>>,
  nodes: Vec<Pin<Box<Node>>>,
}

impl<N: Future<Output = ()>, Node: Future<Output = NodeEvent>>
  SimulationExecutor<N, Node>
{
  pub fn new(network: Pin<Box<N>>) -> Self {
    SimulationExecutor {
      network,
      nodes: Vec::new(),
    }
  }

  pub fn add_node(&mut self, node: Pin<Box<Node>>) {
    self.nodes.push(node);
  }

  pub fn run_tick(&mut self) {
    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    // Attempt to progess the network
    if let Poll::Ready(_) = self.network.as_mut().poll(&mut cx) {
      // The network has completed its operation
      return;
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
