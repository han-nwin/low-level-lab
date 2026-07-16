fn main() {
    // cannot assign twice to immutable variable
    // let x = 5;

    //variables are immutable by default, you can make them mutable
    let mut x = 5;

    println!("The value of x is: {x}");
    x = 6;
    println!("The value of x is: {x}");

    // SHADOWING:
    // Shadowing allows variable transformations while keeping immutability after the transformations.
    // Unlike mutable variables, shadowed variables cannot be reassigned without using the let keyword.
    let y = 10;
    let y = y + 1;

    {
        let y = y * 2;
        println!("The value of y in the inner scope is: {y}");
    }
    println!("The value of y in the outer scope is: {y}");

    // we can also change the type
    let spaces = "   "; // String
    println!("{spaces}");
    let spaces = spaces.len(); // usize
    println!("{spaces}");

    // CONSTANT: Not allow mut
    const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
    println!("{THREE_HOURS_IN_SECONDS}");
}
