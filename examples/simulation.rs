use {
  c2n::{
    node::Node,
    node_config::NodeConfigBuilder,
    simulation_executor::SimulationExecutor,
  },
  futures::Future,
  rand::SeedableRng,
  std::{
    pin::Pin,
    task::{Context, Poll},
  },
};

struct SimStorage;

impl Future for SimStorage {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating storage...");
    Poll::Ready(())
  }
}

impl c2n::storage::Storage for SimStorage {
  fn read(&mut self) -> String {
    String::new()
  }

  fn write(&mut self, message: String) {
    println!("Writing message: {}", message);
  }
}

#[derive(Clone)]
struct SimNetwork;

impl Future for SimNetwork {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating network...");
    Poll::Ready(())
  }
}

impl c2n::network::Network for SimNetwork {
  fn send(&mut self, message: String) {
    println!("Sending message: {}", message);
  }

  fn receive(&mut self) -> Vec<String> {
    vec![]
  }
}

const NODE_COUNT: usize = 10;

fn main() -> anyhow::Result<()> {
  let mut rng = rand::rngs::StdRng::from_seed([0; 32]);

  let mut simulation = SimulationExecutor::new();

  let network = SimNetwork;

  let bootnode_config = NodeConfigBuilder::new()
    .with_unique_identity(&mut rng)
    .build();
  let bootnode = Node::builder()
    .network(network.clone())
    .storage(SimStorage)
    .with_node_config(bootnode_config)
    .build();

  for _ in 0..NODE_COUNT {
    let config = NodeConfigBuilder::new()
      .with_bootnode(bootnode.config().identity)
      .with_unique_identity(&mut rng)
      .build();
    let node = Node::builder()
      .network(network.clone())
      .storage(SimStorage)
      .with_node_config(config)
      .build();
    simulation.add_node(node);
  }

  for _ in 0..10 {
    simulation.run_tick();
  }

  Ok(())
}
