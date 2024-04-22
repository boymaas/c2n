use {
  c2n::{
    network::sim::SimNetwork,
    node::Node,
    node_config::NodeConfigBuilder,
    simulation_executor::SimulationExecutor,
    storage::sim::SimStorage,
  },
  rand::SeedableRng,
};

const NODE_COUNT: usize = 10;

fn main() -> anyhow::Result<()> {
  // Everything we do is based on a seeded random number generator. This makes
  // it possible to simulate many different scenarios independently.
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
