use {
  ::rand::{rngs::StdRng, Rng, SeedableRng},
  c2n::rng::GeneratesRngSeed,
  c2n_simulator::SimBuilder,
  fnv::FnvHashMap,
  macroquad::prelude::*,
};

#[derive(Debug)]
struct Node {
  pos: Vec2,
  vel: Vec2,
}

impl Node {
  fn new(pos: Vec2) -> Self {
    Self {
      pos,
      vel: vec2(0.0, 0.0),
    }
  }
}

#[derive(Debug)]
struct Edge {
  source: usize,
  target: usize,
}

impl Edge {
  fn new(source: usize, target: usize) -> Self {
    Self { source, target }
  }
}

fn edge(source: usize, target: usize) -> Edge {
  Edge::new(source, target)
}

fn apply_forces(nodes: &mut [Node], edges: &[Edge]) {
  let repulsion_strength = 0.05;
  let attraction_strength = 0.1;

  // Apply repulsion force
  for i in 0..nodes.len() {
    for j in 0..nodes.len() {
      if i != j {
        let dx = nodes[j].pos.x - nodes[i].pos.x;
        let dy = nodes[j].pos.y - nodes[i].pos.y;
        let distance = (dx * dx + dy * dy).sqrt().max(1.0); // Avoid division by zero
        let force = repulsion_strength / distance;
        nodes[i].vel.x += dx / distance * force;
        nodes[i].vel.y += dy / distance * force;
      }
    }
  }

  // Apply attraction force
  for edge in edges {
    let source_index = edge.source;
    let target_index = edge.target;
    let dx = nodes[target_index].pos.x - nodes[source_index].pos.x;
    let dy = nodes[target_index].pos.y - nodes[source_index].pos.y;
    let distance = (dx * dx + dy * dy).sqrt().max(1.0);
    let force = attraction_strength * distance - 50.0; // Assuming ideal length of 50
    if force.round() == 0.0 {
      continue;
    }
    nodes[source_index].vel.x += dx / distance * force;
    nodes[source_index].vel.y += dy / distance * force;
    nodes[target_index].vel.x -= dx / distance * force;
    nodes[target_index].vel.y -= dy / distance * force;
  }
}

pub fn random_vec2(rng: &mut impl Rng) -> Vec2 {
  vec2(
    64.0 - rng.gen_range(0.0..128.0),
    64.0 - rng.gen_range(0.0..128.0),
  )
}

#[macroquad::main("Force Directed Graph")]
async fn main() {
  let mut rng = StdRng::seed_from_u64(0);
  let mut simulation = SimBuilder::with_rng(rng.next_rng_seed())
    .with_node_count(50)
    .build();

  let mut pubkeys_index = FnvHashMap::default();

  let mut nodes = vec![];
  let mut spawn_point = vec2(screen_width() / 2.0, screen_height() / 2.0);

  loop {
    // if exscape is pressed, exit the program
    if is_key_pressed(KeyCode::Escape) {
      break;
    }

    clear_background(BLACK);

    // ensure we have and entry in the nodes vec for each nod in de simulation
    for n in 0..simulation.nodes.len() {
      if n >= nodes.len() {
        nodes.push(Node::new(spawn_point + random_vec2(&mut rng)));
        pubkeys_index.insert(*simulation.nodes[n].identity(), n);
      }
    }

    // Lets determine the edges
    let mut edges = vec![];
    for (i, node) in simulation.nodes.iter().enumerate() {
      for connection in node.connections() {
        // we do a fast lookup of pubkey to index
        let connection = pubkeys_index[&connection];
        edges.push(edge(i, connection));
      }
    }

    apply_forces(&mut nodes, &edges);

    // Update node positions based on velocity
    for node in nodes.iter_mut() {
      node.pos.x += node.vel.x.max(-5.0).min(5.0);
      node.pos.y += node.vel.y.max(-5.0).min(5.0);
      node.vel.x *= 0.80; // Damping
      node.vel.y *= 0.80; // Damping
    }

    // Now ensure we zoom so all the nodes are visible
    let mut min_x = 0.0_f32;
    let mut max_x = 0.0_f32;
    let mut min_y = 0.0_f32;
    let mut max_y = 0.0_f32;
    for node in &nodes {
      min_x = min_x.min(node.pos.x);
      max_x = max_x.max(node.pos.x);
      min_y = min_y.min(node.pos.y);
      max_y = max_y.max(node.pos.y);
    }

    let width = max_x - min_x;
    let height = max_y - min_y;
    let center_x = min_x + width / 2.0;
    let center_y = min_y + height / 2.0;
    let zoom = 1.4 / width.min(height);

    // when all these values are numbers
    if width.is_finite()
      && height.is_finite()
      && center_x.is_finite()
      && center_y.is_finite()
      && zoom.is_finite()
    {
      spawn_point = vec2(center_x, center_y);
      // set the camera to the center of the graph
      set_camera(&Camera2D {
        zoom: vec2(zoom, zoom),
        target: vec2(center_x, center_y),
        render_target: None,
        ..Default::default()
      });
    }

    // Draw edges
    for edge in &edges {
      let source = &nodes[edge.source as usize];
      let target = &nodes[edge.target as usize];
      draw_line(
        source.pos.x,
        source.pos.y,
        target.pos.x,
        target.pos.y,
        2.0,
        GRAY,
      );
    }

    // Draw nodes
    for node in &nodes {
      draw_circle(node.pos.x, node.pos.y, 8.0, DARKBLUE);
    }

    simulation.run_tick();

    next_frame().await
  }
}
