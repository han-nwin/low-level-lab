use std::io::Result;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, Builder};

type Job = Box<dyn FnOnce() + Send + 'static>; // since execute() has this trait bound

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(capacity: usize) -> Result<ThreadPool> {
        assert!(capacity > 0);

        let mut workers = Vec::with_capacity(capacity);

        println!("ThreadPool::new : Creating a pair of sender and receiver mutex");
        let (sender, receiver) = mpsc::channel();
        // NOTE: To share ownership across multiple threads and allow the threads to mutate
        // the value, we need to use Arc<Mutex<T>>. The Arc
        // type will let multiple Worker instances own the receiver, and Mutex will ensure that
        // only one Worker gets a job from the receiver at a time.
        let receiver = Arc::new(Mutex::new(receiver));

        // NOTE: instead of spawning thread here, we create a worker to hold a thread
        // So we can access later with the id
        for id in 0..capacity {
            println!("ThreadPool: Creating new worker {}", id);
            workers.push(Worker::new(id, Arc::clone(&receiver))?);
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static, // () mean, the closure f won't take any arg
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap_or_else(|e| {
            panic!("ThreadPool: failed to send job: {}", e);
        });
    }
}

// IMPORTANT:
// SHUTDOWN SEQUENCE IS:
//   main() reaches its end
//           ↓
//   ThreadPool::drop() runs
//           ↓
//   sender.take() drops the sender
//           ↓
//   worker's recv() returns Err
//           ↓
//   worker executes break
//           ↓
//   worker reaches end of its closure
//           ↓
//   actual worker thread terminates
//           ↓
//   join() returns
//           ↓
//   ThreadPool::drop() finishes
//           ↓
//   main() finishes
// Gracefully clean up the thread pool when done
// NOTE: When compiler complains about resource never read -> it's a sign to do Drop
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("ThreadPool::Drop: Dropping the thread pool");

        // 1. First drop the sender so it doesn't send more jobs
        drop(self.sender.take());

        // 2. Then the workers
        // the Vec::drain method.
        // It accepts a range parameter to specify which items to remove from the vector and
        // returns an iterator of those items. Passing the .. range syntax will remove every
        // value from the vector.
        for worker in self.workers.drain(..) {
            println!("ThreadPool::Drop: Shutting down worker {}", worker.id);
            // Note that, because drop can be called when panicking, the unwrap could
            // also panic and cause a double panic, which immediately crashes the program and ends
            // any cleanup in progress.
            // So, we need to use `unwrap_or_else` to handle the error.
            //
            // Wait for the worker thread to finish; then the main thread continues.
            // The worker terminates independently—it does not merge into the main thread.
            worker.thread.join().unwrap_or_else(|e| {
                eprintln!(
                    "ThreadPool::Drop: Error: thread with worker {} could not be joined: {:?}",
                    worker.id, e
                );
            });
        }
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Worker> {
        // spawn new thread
        // using thread::spawn here will panic if system don't have enough thread
        // Builder return a Result<> -> better to handle when fail
        let thread = Builder::new().spawn(move || {
            // worker loop
            // don't use while let Ok(job) = receiver.lock().unwrap().recv() since it will block
            // the mutex -> block the thread
            loop {
                let message = receiver.lock().unwrap().recv();
                // release the lock immediately here
                match message {
                    Ok(job) => {
                        println!("Worker: Worker {} got a job", id);
                        println!("Worker: Worker {} executing job ...", id);
                        job(); // execure the job
                        println!("Worker: Worker {} finished", id);
                    }
                    // NOTE: here is when the thread dropped.
                    // Because if we close sender eventually worker will get here
                    Err(_) => {
                        println!("Worker: Worker {} got an error", id);
                        break; // close the worker loop and terminate
                    }
                }
            }
        })?;

        //result assign to a worker
        Ok(Worker { id, thread })
    }
}
