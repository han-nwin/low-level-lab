use clap::{Parser, Subcommand};

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
}

fn run_atomic(iterations: usize) {
    println!("Atomic version");
}

fn run_aligned(iterations: usize) {
    println!("Aligned version");
}
