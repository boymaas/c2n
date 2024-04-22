use {
  crate::network::Network,
  futures::Future,
  std::{
    pin::Pin,
    task::{Context, Poll},
  },
};

#[derive(Clone)]
pub struct SimNetwork;

impl Future for SimNetwork {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating network...");
    Poll::Ready(())
  }
}

impl Network for SimNetwork {
  fn send(&mut self, message: String) {
    println!("Sending message: {}", message);
  }

  fn receive(&mut self) -> Vec<String> {
    vec![]
  }
}
