use {
  crate::{
    peer_list_manager::PeerListManagerConfig,
    types::{NodeAddress, NodeIdentity, PeerId},
  },
  multiaddr::Multiaddr,
  rand::Rng,
  std::collections::HashSet,
};

pub struct NodeConfig {
  pub bootnodes: HashSet<NodeAddress>,
  pub identity: NodeIdentity,
  pub address: Multiaddr,
  pub peer_list_manager: PeerListManagerConfig,
}

impl NodeConfig {
  pub fn builder() -> NodeConfigBuilder {
    NodeConfigBuilder::new()
  }

  pub fn bootnodes(&self) -> &HashSet<NodeAddress> {
    &self.bootnodes
  }

  pub fn identity(&self) -> &PeerId {
    &self.identity
  }

  pub fn node_address(&self) -> NodeAddress {
    (self.identity, self.address.clone())
  }
}

/// Builder pattern for NodeConfig
pub struct NodeConfigBuilder {
  bootnodes: HashSet<NodeAddress>,
  identity: Option<NodeIdentity>,
  address: Option<Multiaddr>,
  peer_list_manager: PeerListManagerConfig,
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
      address: None,
      peer_list_manager: PeerListManagerConfig::default(),
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

  pub fn with_address(mut self, address: Multiaddr) -> Self {
    self.address = Some(address);
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
      address: self.address.expect("Node address is required"),
      peer_list_manager: PeerListManagerConfig::default(),
    }
  }
}
