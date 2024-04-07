use std::{pin::Pin, task::{Context, Poll}};

use c2n::{node::Node, simulation_executor::SimulationExecutor};
use futures::Future;

struct SimStorage; 

impl Future for SimStorage {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        eprintln!("Simulating storage...");
        Poll::Ready(())
    }
}

impl c2n::storage::Storage for SimStorage {
    fn read(&mut self) -> String {
        String::new()
    }

    fn write(&mut self, message: String) {
        println!("Writing message: {}", message);
    }
}


#[derive(Clone)]
struct SimNetwork;

impl Future for SimNetwork {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        eprintln!("Simulating network...");
        Poll::Ready(())
    }
}

impl c2n::network::Network for SimNetwork {
    fn send(&mut self, message: String) {
        println!("Sending message: {}", message);
    }

    fn receive(&mut self) -> Vec<String> {
        vec![]
    }
}

fn main() -> anyhow::Result<()> {
    let mut simulation = SimulationExecutor::new();

    let network = SimNetwork;

    for _ in 0..10 {
        let storage = SimStorage;
        let network = network.clone();
        simulation.add_node(Node::new(network, storage));
    }

    for _ in 0..10 {
        simulation.run_tick();
    }

    Ok(())
}
