use {
  c2n::{
    network::sim::{SimNetwork, SimNetworkClient, SimNetworkFuture},
    node_config::NodeConfigBuilder,
    peer_list_manager::simple::SimplePeerListManager,
    rng::GeneratesRngSeed,
    simulation_executor::SimulationExecutor,
    storage::sim::SimStorage,
  },
  rand::{Rng, SeedableRng},
  std::{rc::Rc, time::Duration},
};

pub struct SimBuilder<R> {
  rng: R,
  node_count: Option<usize>,
}

impl<R: Rng + SeedableRng + Unpin + 'static> SimBuilder<R> {
  pub fn with_rng(rng: R) -> Self {
    Self {
      rng,
      node_count: None,
    }
  }

  pub fn with_node_count(mut self, node_count: usize) -> Self {
    self.node_count = Some(node_count);
    self
  }

  pub fn build(mut self) -> SimulationExecutor<SimNetworkFuture<R>> {
    let network = SimNetwork::build(self.rng.next_rng_seed());

    let mut simulation =
      SimulationExecutor::new(Box::pin(SimNetworkFuture::wrap(&network)));

    let bootnode_config = NodeConfigBuilder::new()
      .with_unique_identity(&mut self.rng)
      .with_address("/memory/0".parse().unwrap())
      .build();
    let bootnode = c2n::node::Node::builder()
      .network(SimNetworkClient::build(
        self.rng.next_rng_seed(),
        Rc::clone(&network),
        bootnode_config.node_address(),
      ))
      .storage(SimStorage::build(self.rng.next_rng_seed()))
      .peer_list_manager(SimplePeerListManager::build(self.rng.next_rng_seed()))
      .with_node_config(bootnode_config)
      .build();

    // Get the address of the bootnode so that other nodes can connect to it.
    let bootnode_addr = bootnode.config().node_address();

    // We start at 1 second to give the bootnode a head start.
    let mut time_offset = Duration::from_secs(1);
    for idx in 0..self.node_count.expect("node count is required") {
      // Each time a unique identity is generated,
      // the random number generator will be seeded at a new position,
      // giving each node a unique starting sequence.
      let mut rng = self.rng.next_rng_seed();
      let config = NodeConfigBuilder::new()
        .with_bootnode(bootnode_addr.clone())
        .with_unique_identity(&mut rng)
        .with_address(format!("/memory/{}", idx + 1).parse().unwrap())
        .build();
      let node = c2n::node::Node::builder()
        .network(SimNetworkClient::build(
          rng.next_rng_seed(),
          Rc::clone(&network),
          config.node_address(),
        ))
        .storage(SimStorage::build(rng.next_rng_seed()))
        .peer_list_manager(SimplePeerListManager::build(rng.next_rng_seed()))
        .with_node_config(config)
        .build();

      time_offset =
        time_offset + Duration::from_millis(rng.gen_range(100..500));
      simulation.add_node(time_offset, Box::pin(node));
    }

    // add the bootnode to the simulation
    simulation.add_node(Duration::ZERO, Box::pin(bootnode));

    simulation
  }
}
