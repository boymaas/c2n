use {
  crate::types::{NodeAddress, NodeIdentity},
  rand::Rng,
  std::collections::HashSet,
};

pub struct NodeConfig {
  pub bootnodes: HashSet<NodeAddress>,
  pub identity: NodeIdentity,
}

/// Builder pattern for NodeConfig
pub struct NodeConfigBuilder {
  bootnodes: HashSet<NodeAddress>,
  identity: Option<NodeIdentity>,
}

impl Default for NodeConfigBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl NodeConfigBuilder {
  pub fn new() -> Self {
    Self {
      bootnodes: HashSet::new(),
      identity: None,
    }
  }

  pub fn with_bootnode(mut self, bootnode: NodeAddress) -> Self {
    self.bootnodes.insert(bootnode);
    self
  }

  pub fn with_identity(mut self, identity: NodeIdentity) -> Self {
    self.identity = Some(identity);
    self
  }

  pub fn with_unique_identity<R: Rng>(mut self, rng: &mut R) -> Self {
    self.identity = Some(NodeIdentity::unique(rng));
    self
  }

  pub fn build(self) -> NodeConfig {
    NodeConfig {
      bootnodes: self.bootnodes,
      identity: self.identity.expect("Node identity is required"),
    }
  }
}
