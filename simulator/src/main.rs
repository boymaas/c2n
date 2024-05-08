use {
  c2n_simulator::SimBuilder,
  rand::{rngs::StdRng, SeedableRng},
};

const NODE_COUNT: usize = 10;

fn main() -> anyhow::Result<()> {
  // setup tracing with default subscriber
  tracing_subscriber::fmt::init();

  // Everything we do is based on a seeded random number generator. This makes
  // it possible to simulate many different scenarios independently.
  let mut simulation = SimBuilder::with_rng(StdRng::seed_from_u64(0))
    .with_node_count(NODE_COUNT)
    .build();

  loop {
    simulation.run_tick();
  }
}
