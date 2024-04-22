use {
  crate::storage::Storage,
  futures::Future,
  std::{
    pin::Pin,
    task::{Context, Poll},
  },
};

pub struct SimStorage;

impl Future for SimStorage {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    eprintln!("Simulating storage...");
    Poll::Ready(())
  }
}

impl Storage for SimStorage {
  fn read(&mut self) -> String {
    String::new()
  }

  fn write(&mut self, date: String) {
    println!("Writing message: {}", date);
  }
}
