use {
  c2n::{
    network::sim::SimNetwork,
    node::Node,
    node_config::NodeConfigBuilder,
    peer_list_manager::simple::SimplePeerListManager,
    rng::GeneratesRngSeed,
    simulation_executor::SimulationExecutor,
    storage::sim::SimStorage,
  },
  rand::{rngs::StdRng, RngCore, SeedableRng},
  std::{cell::RefCell, rc::Rc},
};

const NODE_COUNT: usize = 10;

fn main() -> anyhow::Result<()> {
  // setup tracing with default subscriber
  tracing_subscriber::fmt::init();

  // Everything we do is based on a seeded random number generator. This makes
  // it possible to simulate many different scenarios independently.
  let mut rng = StdRng::seed_from_u64(0);

  let mut simulation = SimulationExecutor::new();

  let network = SimNetwork::build(rng.clone());

  let bootnode_config = NodeConfigBuilder::new()
    .with_unique_identity(&mut rng)
    .build();
  let bootnode = Node::builder()
    .network(SimNetwork::build(rng.clone()))
    .storage(SimStorage::build(rng.clone()))
    .peer_list_manager(SimplePeerListManager::build(rng.clone()))
    .with_node_config(bootnode_config)
    .build();

  let bootnode_addr = bootnode
    .config()
    .identity()
    .into_node_address("/memory/0".parse().unwrap());

  for idx in 0..NODE_COUNT {
    // Each time a unique identity is generated,
    // the random number generator will be seeded at a new position,
    // giving each node a unique starting sequence.
    let mut rng = rng.next_rng_seed();
    let config = NodeConfigBuilder::new()
      .with_bootnode(bootnode_addr.clone())
      .with_unique_identity(&mut rng)
      .with_address(format!("/memory/{}", idx + 1).parse().unwrap())
      .build();
    let node = Node::builder()
      .network(network.clone())
      .storage(SimStorage::build(rng.next_rng_seed()))
      .peer_list_manager(SimplePeerListManager::build(rng.next_rng_seed()))
      .with_node_config(config)
      .build();
    simulation.add_node(node);
  }

  for _ in 0..10 {
    simulation.run_tick();
  }

  Ok(())
}
