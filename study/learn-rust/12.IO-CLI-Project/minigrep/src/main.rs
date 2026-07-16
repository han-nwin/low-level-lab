use minigrep::{search, search_case_insensitive};
use std::env;
use std::error::Error;
use std::fs;
use std::process;

fn main() {
    // let args: Vec<String> = env::args().collect();

    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}"); // print to stderr
        process::exit(1);
    });
    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);
    println!("--------------");

    if let Err(err) = run(config) {
        eprintln!("Run error: {err}"); // print to stderr
        process::exit(1);
    }
}

struct Config {
    query: String,
    file_path: String,
    ignore_case: bool,
}

impl Config {
    //the returned Config contains string
    // slices borrowed from args, and it cannot outlive
    // args.
    // Use iterator for arg after learning Chapter 13
    fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query String"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok(); // Check if this env var is set

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

// dyn = dynamic dispatch. aka any implementation of error
// box -> since dyn error is unknown size at compile time
// -> box put it on the heap and give us a pointer
fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(config.query.as_str(), &contents)
    } else {
        search(config.query.as_str(), &contents)
    };

    for line in results {
        println!("{line}");
    }

    Ok(())
}
