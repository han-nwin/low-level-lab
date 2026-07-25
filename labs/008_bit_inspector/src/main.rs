use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Inspect { value: String },
    Compare { value1: String, value2: String },
    Add { value1: String, value2: String },
    SetBit { value: String, position: u32 },
    ClearBit { value: String, position: u32 },
    ToggleBit { value: String, position: u32 },
    TestBit { value: String, position: u32 },
    Reverse { value: String },
    Swap { value: String },
    Reinterpret { value: String },
}

fn main() {
    match Cli::parse().command {
        Commands::Inspect { value } => inspect(&value),
        Commands::Compare { value1, value2 } => compare(&value1, &value2),
        Commands::Add { value1, value2 } => add(&value1, &value2),
        // Remaining handlers...
        _ => todo!(),
    }
}

fn inspect(value: &str) {
    println!("Inspecting {}", value);
}

fn compare(value1: &str, value2: &str) {
    println!("Comparing {} and {}", value1, value2);
}

fn add(value1: &str, value2: &str) {
    println!("Adding {} and {}", value1, value2);
}
