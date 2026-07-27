use clap::{Parser, Subcommand};
use std::sync::{Arc, Mutex}; // Arc = Atomic Rc = Atomic reference-counted
use std::thread;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    // if we want command or arg to be optional wrap it in Option
    // command: Option<Commands>,
    // iterations: Option<usize>,
    #[command(subcommand)]
    command: Commands, //required
    #[arg(short, long, default_value_t = 1_000_000)]
    iterations: usize, // required global command
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

fn main() {
    let cli = Cli::parse();

    if cli.iterations == 0 {
        eprintln!("iterations must be greater than zero");
        std::process::exit(1);
    }

    match cli.command {
        Commands::Mutex => run_mutex(cli.iterations),
        Commands::Atomic => run_atomic(cli.iterations),
        Commands::Aligned => run_aligned(cli.iterations),
    }
}

fn run_mutex(iterations: usize) {
    println!("Mutex version");

    let counter = Arc::new(MutexCounter {
        counter_a: Mutex::new(0),
        counter_b: Mutex::new(0),
    });

    let counter_1 = counter.clone();
    let handle_1 = thread::spawn(move || {
        let mut a_cnt = counter_1.counter_a.lock().unwrap();
        *a_cnt += 1; // deref here because mutex implement Deref trait
    });

    let counter_2 = counter.clone();
    let handle_2 = thread::spawn(move || {
        let mut b_cnt = counter_2.counter_b.lock().unwrap();
        *b_cnt += 1; // deref here because mutex implement Deref trait
    });

    handle_1.join().unwrap();
    handle_2.join().unwrap();
}

fn run_atomic(iterations: usize) {
    println!("Atomic version");
}

fn run_aligned(iterations: usize) {
    println!("Aligned version");
}
