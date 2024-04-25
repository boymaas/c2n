use crate::primitives::Pubkey;

pub enum NodeEvent {
  /// The node has successfully dialed and connected to a peer.
  Connected { peer_id: Pubkey },
  /// The node has disconnected from a peer.
  Disconnected { peer_id: Pubkey },
  /// The node has discovered a new peer through the discovery mechanism.
  Discovered { peer_id: Pubkey },
  /// The node has received a message from a peer.
  MessageReceived { peer_id: Pubkey, message: Vec<u8> },
  /// The node has successfully sent a message to a peer.
  MessageSent { peer_id: Pubkey },
  /// The node has entered a new state in the lifecycle.
  StateChanged { new_state: NodeState },
}

pub enum NodeState {
  /// The node is starting up.
  Starting,
  /// The node is running and processing events.
  Running,
  /// The node is shutting down.
  Stopping,
}
