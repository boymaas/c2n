use macroquad::prelude::*;

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
    let source_index = edge.source as usize;
    let target_index = edge.target as usize;
    let dx = nodes[target_index].pos.x - nodes[source_index].pos.x;
    let dy = nodes[target_index].pos.y - nodes[source_index].pos.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let force = attraction_strength * (distance - 50.0); // Assuming ideal length of 50
    nodes[source_index].vel.x += dx / distance * force;
    nodes[source_index].vel.y += dy / distance * force;
    nodes[target_index].vel.x -= dx / distance * force;
    nodes[target_index].vel.y -= dy / distance * force;
  }
}

#[macroquad::main("Force Directed Graph")]
async fn main() {
  let mut nodes = vec![
    Node::new(vec2(400.0, 300.0)),
    Node::new(vec2(450.0, 350.0)),
    Node::new(vec2(350.0, 350.0)),
  ];

  let edges = vec![edge(0, 1), edge(2, 0), edge(1, 2)];

  loop {
    clear_background(BLACK);

    apply_forces(&mut nodes, &edges);

    // Update node positions based on velocity
    for node in nodes.iter_mut() {
      node.pos.x += node.vel.x;
      node.pos.y += node.vel.y;
      node.vel.x *= 0.9; // Damping
      node.vel.y *= 0.9; // Damping
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

    next_frame().await
  }
}
