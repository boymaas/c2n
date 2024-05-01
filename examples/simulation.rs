use {
  c2n::{
    network::sim::{SimNetwork, SimNetworkClient, SimNetworkFuture},
    node::Node,
    node_config::NodeConfigBuilder,
    peer_list_manager::simple::SimplePeerListManager,
    rng::GeneratesRngSeed,
    simulation_executor::SimulationExecutor,
    storage::sim::SimStorage,
  },
  futures::FutureExt,
  rand::{rngs::StdRng, SeedableRng},
  std::rc::Rc,
};

const NODE_COUNT: usize = 10;

fn main() -> anyhow::Result<()> {
  // setup tracing with default subscriber
  tracing_subscriber::fmt::init();

  // Everything we do is based on a seeded random number generator. This makes
  // it possible to simulate many different scenarios independently.

  // TODO: We need to wrap the RNG into a type that is not cloneable.
  // However, we can only use next_rng_seed.
  let mut rng = StdRng::seed_from_u64(0);

  let network = SimNetwork::build(rng.next_rng_seed());

  let mut simulation =
    SimulationExecutor::new(Box::pin(SimNetworkFuture::wrap(&network)));

  let bootnode_config = NodeConfigBuilder::new()
    .with_unique_identity(&mut rng)
    .with_address("/memory/0".parse().unwrap())
    .build();
  let bootnode = Node::builder()
    .network(SimNetworkClient::build(
      rng.next_rng_seed(),
      Rc::clone(&network),
      bootnode_config.node_address(),
    ))
    .storage(SimStorage::build(rng.next_rng_seed()))
    .peer_list_manager(SimplePeerListManager::build(rng.next_rng_seed()))
    .with_node_config(bootnode_config)
    .build();

  // Get the address of the bootnode so that other nodes can connect to it.
  let bootnode_addr = bootnode.config().node_address();

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
      .network(SimNetworkClient::build(
        rng.next_rng_seed(),
        Rc::clone(&network),
        config.node_address(),
      ))
      .storage(SimStorage::build(rng.next_rng_seed()))
      .peer_list_manager(SimplePeerListManager::build(rng.next_rng_seed()))
      .with_node_config(config)
      .build();
    simulation.add_node(Box::pin(node));
  }

  // add the bootnode to the simulation
  simulation.add_node(Box::pin(bootnode));

  loop {
    simulation.run_tick();
  }

  // Ok(())
}
