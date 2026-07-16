use std::pin::Pin;
use std::task::{Context, Poll};

// NOTE:The Stream trait defines an associated type called
// Item for the type of the items produced by the stream.
// This is similar to Iterator, where there may be zero to many items,
// and unlike Future, where there is always a single Output, even if it’s the unit type ().
trait Stream {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}

// NOTE:
// StreamExt is a helper trait that makes streams easier to use.
// Instead of manually working with the low-level poll_next method, you can use next from StreamExt and await it.
// The main idea is:
// Stream is the low-level trait you implement.
// StreamExt gives extra helper methods automatically to anything that implements Stream.
// So if you build your own stream, you only need to implement the basic Stream behavior.
// After that, users can call convenient methods like next without you writing them yourself.
// In short: StreamExt is the nice user-friendly layer on top of Stream.
trait StreamExt: Stream {
    async fn next(&mut self) -> Option<Self::Item>
    where
        Self: Unpin;

    // other methods...
}
// In short: we can use next() instead of using poll_next()
// So we don't have to deal with Pin and Context in poll_next()

fn main() {
    println!("Hello, world!");
}
