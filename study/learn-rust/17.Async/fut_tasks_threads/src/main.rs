use std::{thread, time::Duration};
use tokio::sync::broadcast;

// NOTE:
// Threads and async are both ways to do concurrency.
// Use threads when the work is CPU-heavy, like video encoding or processing lots of data.
// Use async when the work is I/O-heavy, like waiting for network messages, files, timers, or user input.
// They are not enemies. You can use them together.
// Simple idea:
// Threads = good for parallel CPU work.
// Async tasks = good for waiting on many things at once.
// A task is like a lightweight thread managed by the async runtime, not by the OS.
// Futures run inside tasks. The runtime can move tasks between threads to use the CPU better.
fn main() {
    let (tx, mut rx) = trpl::channel();
    let tx_1 = tx.clone();

    // processing data and send it through channel from a different thread
    thread::spawn(move || {
        for i in 1..11 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });
    thread::spawn(move || {
        for i in 11..20 {
            tx_1.send(i).unwrap();
            thread::sleep(Duration::from_secs(2));
        }
    });

    // receiving data on an async task
    trpl::block_on(async {
        while let Some(message) = rx.recv().await {
            println!("{message}");
        }
    });

    // Test tokio broadcast
    let (tx, mut rx) = broadcast::channel(10);
    let mut rx_1 = tx.subscribe();
    thread::spawn(move || {
        for i in 1..11 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });
    trpl::block_on(async {
        while let Ok(message) = rx.recv().await {
            println!("{message}");
        }
    });
    trpl::block_on(async {
        while let Ok(message) = rx_1.recv().await {
            println!("{message}");
        }
    });
}
