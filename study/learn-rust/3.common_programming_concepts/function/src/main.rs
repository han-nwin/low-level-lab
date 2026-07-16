fn main() {
    println!("Hello, world!");

    another_function(5);

    // Statements and Expressions
    // Statements are instructions that perform an action and do not return a value.
    // Expressions are values that can be used in an expression.

    // Statements
    let x = 5;
    let y = 10;

    if x > y {
        println!("x is greater than y");
    }

    // Expressions
    let k = {
        let x = 3;
        x + 1
        // Expressions do not include ending semicolons.
        // If you add a semicolon to the end of an expression, you turn it into a statement,
        // and it will then not return a value.
        // Keep this in mind as you explore function return values and expressions next.
    };

    println!("k is {}", k);

    let five = five();
    println!("five = {five}");

    let plus = plus_one(5);
    println!("plus = {plus}");
}

fn another_function(x: i32) {
    println!("Another function. {x}");
}

// Function with return value
fn five() -> i32 {
    5
}

fn plus_one(x: i32) -> i32 {
    x + 1
}
