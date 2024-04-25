// Re-export `Pubkey` from the `primitives` module for external use.
pub use crate::primitives::Pubkey;

// `PeerId` is a type alias for a public key that uniquely identifies a peer in
// the network.
pub type PeerId = Pubkey;

// `PeerReputation` is an integer score representing the reputation of a peer
// based on its behavior.
pub type PeerReputation = i32;

// `NodeAddress` is a type alias for a public key representing the address of a
// node in the network.
pub type NodeAddress = Pubkey;

// `NodeIdentity` is a type alias for a public key used to identify a node
// uniquely.
pub type NodeIdentity = Pubkey;

