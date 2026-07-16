use std::rc::Rc;
use std::sync::{Arc, Mutex}; // Arc = Atomic Rc = Atomic reference-counted
use std::thread;

fn main() {
    let m = Mutex::new(5);

    {
        let mut num = m.lock().unwrap();
        *num = 6; // defef here since mutex inplement DeRef trait
    }

    println!("m = {m:?}");

    // === Multi threads with mutex === //
    let counter = Arc::new(Mutex::new(0)); // Atomic reference-counted to the Mutex<T>
    let mut handles = vec![];

    for _ in 0..10 {
        let cnt = Arc::clone(&counter); // create a clone before sending it to thread

        let handle = thread::spawn(move || {
            let mut num = cnt.lock().unwrap();
            *num += 1; //defef here since mutex inplement DeRef trait
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Result: {}", *counter.lock().unwrap());
}

// NOTE:
// So if normal Rc<RefCell<>> can cause mem leaks with reference cycles
// Arc<Mutex>> can cause deadlocks with reference cycles
