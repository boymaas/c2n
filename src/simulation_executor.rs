use {
  crate::{network::Network, node::Node, storage::Storage},
  futures::Future,
  std::{pin::Pin, task::Context},
};

pub struct SimulationExecutor {
  nodes: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Default for SimulationExecutor {
  fn default() -> Self {
    Self::new()
  }
}

impl SimulationExecutor {
  pub fn new() -> Self {
    SimulationExecutor { nodes: Vec::new() }
  }

  pub fn add_node<N, S>(&mut self, node: Node<N, S>)
  where
    N: Network + Unpin + 'static,
    S: Storage + Unpin + 'static,
  {
    self.nodes.push(Box::pin(node));
  }

  pub fn run_tick(&mut self) {
    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    // Attempt to progress each node's state by one tick.
    for node in &mut self.nodes {
      let _ = node.as_mut().poll(&mut cx);
    }
  }

  // Here you would handle any logic to check if the simulation should continue
  // or if specific nodes have completed their operations.
}
