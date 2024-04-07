pub mod memory;

use std::future::Future;

/// A network interface that can send and receive messages.

pub trait Network: Future<Output = ()> {
    fn send(&mut self, message: String);
    fn receive(&mut self) -> Vec<String>;
}

