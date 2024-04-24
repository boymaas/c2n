use {
  crate::network::Network,
  futures::Future,
  rand::{rngs::StdRng, Rng},
  std::{
    pin::Pin,
    task::{Context, Poll},
  },
};

#[derive(Clone)]
pub struct SimNetwork<R> {
  rng: R,
}

impl<R> Future for SimNetwork<R> {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating network...");
    Poll::Ready(())
  }
}

impl<R> Network for SimNetwork<R> {
  fn send(&mut self, message: String) {
    println!("Sending message: {}", message);
  }

  fn receive(&mut self) -> Vec<String> {
    vec![]
  }
}

impl<R: Rng> SimNetwork<R> {
  pub fn build(rng: R) -> Self {
    SimNetwork { rng }
  }
}
