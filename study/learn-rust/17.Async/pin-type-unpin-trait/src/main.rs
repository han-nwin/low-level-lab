//NOTE:
// Pin is a wrapper for pointer-like types such as &, &mut,
// Box, and Rc. (Technically, Pin works with types that implement the
// Deref or DerefMut traits, but this is effectively equivalent to working only with references
// and smart pointers.) Pin is not a pointer itself and doesn’t have
// any behavior of its own like Rc and Arc do with reference counting; it’s
// purely a tool the compiler can use to enforce constraints on pointer usage.
//
// NOTE:
// When we move a future—whether by pushing it into a data structure to
// use as an iterator with join_all or by returning it from a function—that
// actually means moving the state machine Rust creates for us. And unlike most other
// types in Rust, the futures Rust creates for async blocks can end up with
// references to themselves in the fields of any given variant,
//
// NOTE:
// Theoretically, the Rust compiler could try to update every reference to an object whenever
// it gets moved, but that could add a lot of performance overhead, especially
// if a whole web of references needs updating. If we could instead make sure
// the data structure in question doesn’t move in memory, we wouldn’
// t have to update any references. This is exactly what Rust’s borrow
// checker is for: in safe code, it prevents you from moving any item with an active reference to it.

// NOTE:
// Pin builds on that to give us the exact guarantee we need. When we
// pin a value by wrapping a pointer to that value in Pin, it can
// no longer move. Thus, if you have Pin<Box<SomeType>>,
// you actually pin the SomeType value, not the Box pointer.

use std::pin::{Pin, pin};
use std::time::Duration;

fn main() {
    trpl::block_on(async {
        let (tx, mut rx) = trpl::channel();

        let tx1 = tx.clone();
        let tx1_fut = pin!(async move {
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("future"),
            ];

            for val in vals {
                tx1.send(val).unwrap();
                trpl::sleep(Duration::from_secs(1)).await;
            }
        });
        let rx_fut = pin!(async {
            while let Some(value) = rx.recv().await {
                println!("received '{value}'");
            }
        });
        let tx_fut = pin!(async move {
            let vals = vec![
                String::from("more"),
                String::from("messages"),
                String::from("for"),
                String::from("you"),
            ];

            for val in vals {
                tx.send(val).unwrap();
                trpl::sleep(Duration::from_secs(1)).await;
            }
        });
        let futures: Vec<Pin<&mut dyn Future<Output = ()>>> = vec![tx1_fut, rx_fut, tx_fut];

        trpl::join_all(futures).await;
    });
}
