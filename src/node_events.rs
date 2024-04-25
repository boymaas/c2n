use crate::primitives::Pubkey;

pub enum NodeEvent {
  /// The node has successfully dialed and connected to a peer.
  PeerConnected { peer_id: Pubkey },
  /// The node has disconnected from a peer.
  PeerDisconnected { peer_id: Pubkey },
  /// The node has discovered a new peer through the discovery mechanism.
  Discovered { peer_id: Pubkey },
  /// The node has entered a new state in the lifecycle.
  StateChanged { new_state: NodeState },
  /// Noop event to return from the future and let the runtime
  /// poll the future again
  Noop,
}

pub enum NodeState {
  /// The node is starting up.
  Starting,
  /// The node is running and processing events.
  Running,
  /// The node is shutting down.
  Stopping,
}
