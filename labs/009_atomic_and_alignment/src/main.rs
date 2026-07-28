use clap::{Parser, Subcommand};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex}; // Arc = Atomic Rc = Atomic reference-counted
use std::thread;
use std::time::Instant;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    // if we want command or arg to be optional wrap it in Option
    // command: Option<Commands>,
    // iterations: Option<usize>,
    #[command(subcommand)]
    command: Commands, //required
    #[arg(short, long, default_value_t = 1_000_000, global = true)] // make this global
    iterations: usize, // required global flag
}

#[derive(Subcommand)]
enum Commands {
    Mutex,
    Atomic,
    Aligned,
    // if we want a flag to be under a command do
    // Mutex {
    //     #[arg(short, long, default_value_t = 1_000_000)]
    //     iterations: usize, // local command
    // },
}

struct MutexCounter {
    counter_a: Mutex<u64>,
    counter_b: Mutex<u64>,
}

// NOTE: One 64-byte cache line
// ┌───────────────────────────────────────────────┐
// │ a: 8 bytes (aka 64bit) │ b: 8 bytes │ unused: 58 bytes  │
// └───────────────────────────────────────────────┘
struct AtomicCounter {
    counter_a: AtomicU64,
    counter_b: AtomicU64,
}

// NOTE: repr = “Represent this type in memory using these layout rules.”
#[repr(align(64))]
struct AlignedCounter {
    value: AtomicU64,
}

// cache line 1:
// ┌──────────────────────────────────────────────┐
// │ counter_a: 8 B │ padding: 56 B              │
// └──────────────────────────────────────────────┘
//
// cache line 2:
// ┌──────────────────────────────────────────────┐
// │ counter_b: 8 B │ padding: 56 B              │
// └──────────────────────────────────────────────┘
struct AlignedCounters {
    counter_a: AlignedCounter,
    counter_b: AlignedCounter,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Mutex => run_mutex(cli.iterations),
        Commands::Atomic => run_atomic(cli.iterations),
        Commands::Aligned => run_aligned(cli.iterations),
    }

    if cli.iterations == 0 {
        eprintln!("iterations must be greater than zero");
        std::process::exit(1);
    }
}

fn run_mutex(iterations: usize) {
    println!("Mutex version:");

    let time = Instant::now();

    let counter = Arc::new(MutexCounter {
        counter_a: Mutex::new(0),
        counter_b: Mutex::new(0),
    });

    let counter_1 = counter.clone(); // make a atomic reference
    let handle_1 = thread::spawn(move || {
        for _ in 1..=iterations {
            let mut a_cnt = counter_1.counter_a.lock().unwrap();
            *a_cnt += 1; // deref here because mutex implement Deref trait
        }
    });

    let counter_2 = counter.clone();
    let handle_2 = thread::spawn(move || {
        for _ in 1..=iterations {
            let mut b_cnt = counter_2.counter_b.lock().unwrap();
            *b_cnt += 1; // deref here because mutex implement Deref trait
        }
    });

    handle_1.join().unwrap();
    handle_2.join().unwrap();

    println!("Iterations: {iterations}");
    println!("Counter A: {}", counter.counter_a.lock().unwrap());
    println!("Counter B: {}", counter.counter_b.lock().unwrap());
    println!("Elapsed: {} milli seconds", time.elapsed().as_millis());
    println!(
        "Operations/second: {:.02}",
        iterations as f64 / time.elapsed().as_secs_f64()
    );
}

// Relaxed = atomic only
// Release = send/publish
// Acquire = receive/observe
// AcqRel  = receive + send
// SeqCst  = strongest, one obvious global order
fn run_atomic(iterations: usize) {
    println!("Atomic version:");

    let time = Instant::now();

    let counter = Arc::new(AtomicCounter {
        counter_a: AtomicU64::new(0),
        counter_b: AtomicU64::new(0),
    });

    let counter_1 = counter.clone();
    let handle_1 = thread::spawn(move || {
        for _ in 1..=iterations {
            counter_1.counter_a.fetch_add(1, Ordering::Relaxed);
        }
    });

    let counter_2 = counter.clone();
    let handle_2 = thread::spawn(move || {
        for _ in 1..=iterations {
            counter_2.counter_b.fetch_add(1, Ordering::Relaxed);
        }
    });

    handle_1.join().unwrap();
    handle_2.join().unwrap();

    println!("Iterations: {iterations}");
    println!("Counter A: {}", counter.counter_a.load(Ordering::Relaxed));
    println!("Counter B: {}", counter.counter_b.load(Ordering::Relaxed));
    println!("Elapsed: {} milli seconds", time.elapsed().as_millis());
    println!(
        "Operations/second: {:.02}",
        iterations as f64 / time.elapsed().as_secs_f64()
    );
}

fn run_aligned(iterations: usize) {
    println!("Aligned version:");

    let time = Instant::now();

    let counter = Arc::new(AlignedCounters {
        counter_a: AlignedCounter {
            value: AtomicU64::new(0),
        },
        counter_b: AlignedCounter {
            value: AtomicU64::new(0),
        },
    });

    let counter_1 = counter.clone();
    let handle_1 = thread::spawn(move || {
        for _ in 1..=iterations {
            counter_1.counter_a.value.fetch_add(1, Ordering::Relaxed);
        }
    });

    let counter_2 = counter.clone();
    let handle_2 = thread::spawn(move || {
        for _ in 1..iterations {
            counter_2.counter_b.value.fetch_add(1, Ordering::Relaxed);
        }
    });

    handle_1.join().unwrap();
    handle_2.join().unwrap();

    println!("Iterations: {iterations}");
    println!(
        "Counter A: {}",
        counter.counter_a.value.load(Ordering::Relaxed)
    );
    println!(
        "Counter B: {}",
        counter.counter_b.value.load(Ordering::Relaxed)
    );
    println!("Elapsed: {} milli seconds", time.elapsed().as_millis());
    println!(
        "Operations/second: {:.02}",
        iterations as f64 / time.elapsed().as_secs_f64()
    );
}
