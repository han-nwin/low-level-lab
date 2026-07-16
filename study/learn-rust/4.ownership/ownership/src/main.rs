fn main() {
    // The String Type
    let mut s = String::from("hello"); // create a string from a string literal

    s.push_str(", world!"); // push_str() appends a literal to a String

    println!("{s}"); // this will print `hello, world!`

    compare();

    function_ownership();
    function_ownership_2();
    function_ownership_3();
} // this scope is now over, and s is no longer valid. drop() is called

fn compare() {
    //======= STACK ===========//
    // because integers are simple values with a known,
    // fixed size, and these two 5 values are pushed onto the stack.
    let x: i32 = 5;
    let y = x;

    //======= HEAP ===========//
    // this is different
    // this is transferring the ownership of pointer poiting to the address of the first value of the memory block
    // on the heep. aka a move (not a copy)
    let s1 = String::from("hello");
    let s2 = s1; // Since now s1 is no longer valid
    // println!("{s1}, world!"); // this will cause compile error.
    println!("{s2}, Han!"); // this will cause compile error.

    // When you assign a completely new value to an existing variable, Rust will call drop and free the original value’s memory immediately.
    let mut s = String::from("hello");
    s = String::from("ahoy"); // [hello] got freed here by drop() 

    println!("{s}, world!");

    // A Deep copy would use clone(). This is expensive since data and pointer are duplicated
    let s1 = String::from("hello");
    let s2 = s1.clone();

    println!("s1 = {s1}, s2 = {s2}");

    // NOTES:
    // Rust has a special annotation called the Copy trait that we can place on types that are stored on the STACK
    // If a type, or any of its parts, has implemented the Drop trait, Rust won't allow to create
    // Copy trait
    //
    // Check the doc of the type to see if it has Copy trait or not
    //
    // Example:
    //All the integer types, such as u32.
    // The Boolean type, bool, with values true and false.
    // All the floating-point types, such as f64.
    // The character type, char.
    // Tuples, if they only contain types that also implement Copy. For example, (i32, i32) implements Copy, but (i32, String) does not

    let c = [1, 2, 3];
    let d = c;
    println!("{:?}", c);

    let e = (1, 2, 3);
    let f = e;
    println!("{:?}", e);
}

fn function_ownership() {
    let s = String::from("hello"); // s comes into scope

    takes_ownership(s); // s's value moves into the function...
    // ... and so is no longer valid here
    // println!("{s}"); // This will throw error since s is drop in takes_ownership

    let x = 5; // x comes into scope

    makes_copy(x); // Because i32 implements the Copy trait,
    // x does NOT move into the function,
    // so it's okay to use x afterward.
} // Here, x goes out of scope, then s. However, because s's value was moved, nothing special happens.

fn takes_ownership(some_string: String) {
    // some_string comes into scope
    println!("{some_string}");
} // Here, some_string goes out of scope and `drop` is called. The backing
// memory is freed.

fn makes_copy(some_integer: i32) {
    // some_integer comes into scope
    println!("{some_integer}");
} // Here, some_integer goes out of scope. Nothing special happens.

//== Trick to get back ownership after passing to function (return it back) ===//
// Returning values can also transfer ownership.
fn function_ownership_2() {
    let s1 = gives_ownership(); // gives_ownership moves its return
    // value into s1

    let s2 = String::from("hello function ownership 2"); // s2 comes into scope

    let s3 = takes_and_gives_back(s2); // s2 is moved into
    // takes_and_gives_back, which also
    // moves its return value into s3
    println!("{s3}");
} // Here, s3 goes out of scope and is dropped. s2 was moved, so nothing
// happens. s1 goes out of scope and is dropped.

fn gives_ownership() -> String {
    // gives_ownership will move its
    // return value into the function
    // that calls it

    let some_string = String::from("yours"); // some_string comes into scope

    some_string // some_string is returned and
    // moves out to the calling
    // function
}

// This function takes a String and returns a String.
fn takes_and_gives_back(a_string: String) -> String {
    // a_string comes into
    // scope

    a_string // a_string is returned and moves out to the calling function
}

// Return multiple values/ownerships using tuple
fn function_ownership_3() {
    let s1 = String::from("hello function ownerships 3");

    let (s2, len) = calculate_length(s1);

    println!("The length of '{s2}' is {len}.");
}

fn calculate_length(s: String) -> (String, usize) {
    let length = s.len(); // len() returns the length of a String

    (s, length)
}
