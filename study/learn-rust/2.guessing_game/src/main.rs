use std::cmp::Ordering;
use std::io;

use rand::RngExt;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::rng().random_range(1..100);

    // println!("The secret number is: {secret_number}");

    loop {
        println!("Please input your guess.");

        let mut guess = String::new(); // mutable var

        // Read user input
        // Result's variants are Ok and Err. If Ok -> take the output of Ok
        // if Err -> need to handle error
        // The right way to suppress the warning
        // is to actually write error-handling code,
        // but in our case we just want to crash this program when a problem occurs,
        // so we can use expect
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        // type conversion to unsigned 32-bit number
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        println!("You guessed: {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big"),
            Ordering::Equal => {
                println!("You win!");
                break; //if correct get out of loop
            }
        }
    }
}
