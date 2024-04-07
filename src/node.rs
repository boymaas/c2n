use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::ready;

use crate::network::Network;
use crate::storage::Storage;

pub struct Node<N, S>
where
    N: Network,
    S: Storage,
{
    network: N,
    storage: S,
}

impl<N, S> Node<N, S>
where
    N: Network,
    S: Storage,
{
    pub fn new(network: N, storage: S) -> Self {
        Self { network, storage }
    }
}

impl<N, S> Future for Node<N, S>
where
    N: Network + Unpin,
    S: Storage + Unpin,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        // Example of progressing network and storage components.
        // Here, you'd likely manage state transitions or operations
        // that need to happen over multiple ticks.
        
        // For demonstration, assume we simply call poll on both
        // components to simulate progressing their states.
        ready!(Pin::new(&mut this.network).poll(cx));
        ready!(Pin::new(&mut this.storage).poll(cx));

        // This example does not complete, to illustrate ongoing processing.
        // Adjust according to your simulation's end conditions.
        Poll::Pending
    }
}
