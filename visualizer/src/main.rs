use {
  ::rand::{rngs::StdRng, SeedableRng},
  c2n::types::PeerId,
  c2n_simulator::SimBuilder,
  macroquad::prelude::*,
  std::{collections::HashMap, f32},
};

struct VisNode {
  pos: Vec2,
  color: Color,
  peer_id: PeerId,
}

impl VisNode {
  fn new(pos: Vec2, color: Color, peer_id: PeerId) -> Self {
    Self {
      pos,
      color,
      peer_id,
    }
  }

  fn draw(&self) {
    draw_circle(self.pos.x, self.pos.y, 10.0, self.color);
    // draw_text(
    //   &self.peer_id.to_string(),
    //   self.pos.x + 15.0,
    //   self.pos.y + 5.0,
    //   15.0,
    //   WHITE,
    // );
  }
}

fn draw_nodes(nodes: &[VisNode], connections: &HashMap<PeerId, Vec<PeerId>>) {
  clear_background(BLACK);

  for node in nodes {
    node.draw();
  }

  for (peer_id, connected_peers) in connections {
    if let Some(node) = nodes.iter().find(|n| n.peer_id == *peer_id) {
      for connected_peer in connected_peers {
        if let Some(other_node) =
          nodes.iter().find(|n| n.peer_id == *connected_peer)
        {
          draw_line(
            node.pos.x,
            node.pos.y,
            other_node.pos.x,
            other_node.pos.y,
            1.0,
            WHITE,
          );
        }
      }
    }
  }
}

fn generate_node_positions(
  num_nodes: usize,
  screen_width: f32,
  screen_height: f32,
) -> Vec<Vec2> {
  let radius = f32::min(screen_width, screen_height) * 0.4;
  let center_x = screen_width / 2.0;
  let center_y = screen_height / 2.0;

  (0..num_nodes)
    .map(|i| {
      let angle = f32::consts::PI * 2.0 * i as f32 / num_nodes as f32;
      vec2(
        center_x + angle.cos() * radius,
        center_y + angle.sin() * radius,
      )
    })
    .collect()
}

#[macroquad::main("Node Visualization")]
async fn main() {
  let mut simulation = SimBuilder::with_rng(StdRng::seed_from_u64(0))
    .with_node_count(50)
    .build();

  let node_colors = [RED, GREEN, BLUE, YELLOW, MAGENTA, ORANGE, PURPLE];

  loop {
    let screen_width = screen_width();
    let screen_height = screen_height();

    let peer_ids: Vec<_> = simulation
      .nodes
      .iter()
      .map(|n| n.identity())
      .cloned()
      .collect();
    let num_nodes = peer_ids.len();

    let node_positions =
      generate_node_positions(num_nodes, screen_width, screen_height);

    let mut nodes = Vec::new();
    for (i, peer_id) in peer_ids.into_iter().enumerate() {
      nodes.push(VisNode::new(
        node_positions[i],
        node_colors[i % node_colors.len()],
        peer_id,
      ));
    }

    let connections: HashMap<_, _> = simulation
      .nodes
      .iter()
      .map(|node| (*node.identity(), node.connections()))
      .collect();

    draw_nodes(&nodes, &connections);

    let total_connections: usize = connections.values().map(|v| v.len()).sum();
    let stats_text = format!("Total connections: {}", total_connections);
    draw_text(&stats_text, 10.0, screen_height - 20.0, 20.0, WHITE);

    simulation.run_tick();

    next_frame().await
  }
}
