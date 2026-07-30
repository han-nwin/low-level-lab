use std::thread::available_parallelism;

fn main() -> std::io::Result<()> {
    let count = available_parallelism()?.get();
    println!("Number of available threads is {}", count);
    Ok(())
}
