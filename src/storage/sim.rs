use {
  crate::storage::Storage,
  futures::Future,
  std::{
    pin::Pin,
    task::{Context, Poll},
  },
};

pub struct SimStorage<R> {
  rng: R,
}

impl<R> Future for SimStorage<R> {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating storage...");
    Poll::Ready(())
  }
}

impl<R> Storage for SimStorage<R> {
  fn read(&mut self) -> String {
    String::new()
  }

  fn write(&mut self, date: String) {
    println!("Writing message: {}", date);
  }
}

impl<R> SimStorage<R> {
  pub fn build(rng: R) -> Self {
    SimStorage { rng }
  }
}
