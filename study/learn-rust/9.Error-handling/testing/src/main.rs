use std::error::Error;
use std::fs::File;

// dyn = dynamic dispatch. Aka any implementation of Error
// Box -> since dyn Error is unknown size at compile time
// -> Box put it on the heap and give us a pointer
fn main() -> Result<(), Box<dyn Error>> {
    let greeting_file = File::open("hello.txt")?;

    Ok(())
}
