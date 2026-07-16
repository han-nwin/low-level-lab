use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("hi number {i} from the spawned thread!");
            thread::sleep(Duration::from_millis(1));
        }
    });

    // don't join here

    for i in 1..5 {
        println!("hi number {i} from the main thread!");
        thread::sleep(Duration::from_millis(1));
    }

    handle
        .join()
        .unwrap_or_else(|e| eprintln!("Error: {:?}", e));

    let v = vec![1, 2, 3];

    // move v ownership to this thread
    let handle2 = thread::spawn(move || {
        println!("Here's a vector: {v:?}");
    });

    handle2
        .join()
        .unwrap_or_else(|e| eprintln!("Error: {:?}", e));

    // println!("vec: {:?}", v); //this won't work since it's already moved
    //

    // === Channel === //
    println!("=== Channel ===");

    // sending data between threads
    // tx: transmitter
    // rx: receiver
    let (tx, rx) = mpsc::channel(); // mpsc = multi producers, single consumer
    let tx1 = tx.clone(); // clone another transmitter

    // Producer 1
    thread::spawn(move || {
        let vals = vec![
            String::from("Hello from thread!"),
            String::from("This is 1st message"),
            String::from("This is 2nd msg"),
            String::from("This is 3rd msg"),
        ];
        for val in vals {
            tx.send(val).unwrap_or_else(|e| eprintln!("Error: {:?}", e));
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Producer 2
    thread::spawn(move || {
        let vals = vec![
            String::from("more"),
            String::from("messages"),
            String::from("for"),
            String::from("you"),
        ];

        for val in vals {
            tx1.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    for receive in rx {
        println!("Got: {receive}");
    }
}
