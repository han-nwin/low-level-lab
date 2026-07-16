use std::thread;
use std::time::{Duration, Instant};

// Expensive function
fn calculate(s: u64) {
    let mut x: u64 = 1_000;
    let end = Instant::now() + Duration::from_secs(s);
    while Instant::now() < end {
        x = x.wrapping_add(x.wrapping_mul(2));
    }
    println!("Calculate done");
    std::hint::black_box(x); // avoid compiler optimization
}

// Real work function
fn calculate_work(iterations: u64) -> u64 {
    let mut x: u64 = 1_000;

    for _ in 0..iterations {
        x = x.wrapping_add(x.wrapping_mul(2));
    }

    std::hint::black_box(x) // avoid compiler optimization
}

// NOTE:
// testing machine has 8 threads, 8 cores
// This compares where work runs.
// It does not mean tokio::spawn is the right tool for CPU-heavy loops.
#[tokio::main]
async fn main() {
    const TASKS: u64 = 30;
    const WORK_PER_TASK: u64 = 1_000_000_000;
    const TOTAL_WORK: u64 = TASKS * WORK_PER_TASK;

    println!("== Thread ==");
    let start = Instant::now();
    let mut handles = vec![];
    for i in 1..=TASKS {
        let handle = thread::spawn(move || {
            println!("Starting task {}", i);
            // calculate(WORKS);
            calculate_work(WORK_PER_TASK);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Thread Time took: {} secs", start.elapsed().as_secs()); // equivalent to now - start
    println!(
        "throughput: {:.2} iterations/sec",
        TOTAL_WORK as f64 / start.elapsed().as_secs_f64()
    );

    println!("=== Tokio spawn_blocking ===");
    let start = Instant::now();
    let mut handles = vec![];
    for i in 1..=TASKS {
        let handle = tokio::task::spawn_blocking(move || {
            println!("Starting task {}", i);
            calculate_work(WORK_PER_TASK);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }

    println!(
        "Tokio spawn_blocking Time took: {} secs",
        start.elapsed().as_secs()
    );
    println!(
        "throughput: {:.2} iterations/sec",
        TOTAL_WORK as f64 / start.elapsed().as_secs_f64()
    );

    // NOTE: we will see that this will create 8 tasks and finish then create another 8 ...
    println!("=== Tokio spawn===");
    let start = Instant::now();
    let mut handles = vec![];
    for i in 1..=TASKS {
        let handle = tokio::spawn(async move {
            println!("Starting task {}", i);
            calculate_work(WORK_PER_TASK)
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }

    println!("Tokio spawn Time took: {} secs", start.elapsed().as_secs());

    println!(
        "throughput: {:.2} iterations/sec",
        TOTAL_WORK as f64 / start.elapsed().as_secs_f64()
    );
}
