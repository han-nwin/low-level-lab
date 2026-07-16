use std::io;

fn main() {
    // 1. SCALAR TYPES
    // 1. SCALAR TYPES
    //
    // 1.1 Integer
    // | Length                | Signed | Unsigned |
    // |-----------------------|--------|----------|
    // | 8-bit                 | i8     | u8       |
    // | 16-bit                | i16    | u16      |
    // | 32-bit                | i32    | u32      |
    // | 64-bit                | i64    | u64      |
    // | 128-bit               | i128   | u128     |
    // | Architecture-dependent | isize  | usize    |

    // number literal
    //Number literals	Example
    // Decimal	98_222
    // Hex	0xff
    // Octal	0o77
    // Binary	0b1111_0000
    // Byte (u8 only)	b'A'

    //  1.2 Floating point
    //  f32 or f64. Default is f64
    let x = 2.0; // f64
    let y: f32 = 3.0; // f32

    // Operations
    // addition
    let sum = 5 + 10;

    // subtraction
    let difference = 95.5 - 4.3;

    // multiplication
    let product = 4 * 30;

    // division
    let quotient = 56.7 / 32.2;
    let truncated = -5 / 3; // Results in -1

    // remainder
    let remainder = 43 % 5;

    println!("x: {}", x);
    println!("y: {}", y);
    println!("sum: {}", sum);
    println!("difference: {}", difference);
    println!("product: {}", product);
    println!("quotient: {}", quotient);
    println!("truncated: {}", truncated);
    println!("remainder: {}", remainder);

    // 1.3 Boolean
    let t = true;
    let f: bool = false; // with explicit type annotation

    // 1.4 Characters: 4-bytes Unicode scalar value
    let c = 'z';
    let z: char = 'ℤ'; // with explicit type annotation
    let heart_eyed_cat = '😻';

    println!("t: {}", t);
    println!("f: {}", f);
    println!("c: {}", c);
    println!("z: {}", z);
    println!("heart_eyed_cat: {}", heart_eyed_cat);

    // 2. COMPOUND TYPES: Group multiple types into 1 type
    //  2.1 Tuple
    // a general way of grouping together a number of values with a variety of types into one compound type.
    // Tuples have a fixed length: Once declared, they cannot grow or shrink in size.
    let tup: (i32, f64, u8) = (500, 6.4, 1);
    println!("tup: {:?}", tup); // the :? syntax print debug mode aka tup has Debug() implemented
    // Get single value out of the tup
    let tup = (500, 6.4, 1);

    let (x, y, z) = tup;

    println!("The value of y is: {y}");

    // Access tuple elements with . following with the index
    let x: (i32, f64, u8) = (500, 6.4, 1);

    let five_hundred = x.0;

    let six_point_four = x.1;

    let one = x.2;
    println!("five_hundred: {}", five_hundred);
    println!("six_point_four: {}", six_point_four);
    println!("one: {}", one);

    // 2.2 Arrays
    // Another way to have a collection of multiple values is with an array.
    // NOTES: Unlike a tuple, every element of an array must have the same type.
    // NOTES: Unlike arrays in some other languages, arrays in Rust have a fixed length.
    let a = [1, 2, 3, 4, 5];

    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let b: [i32; 5] = [1, 2, 3, 4, 5];

    let first = a[0];
    let second = a[1];
    println!("Please enter an array index.");

    let mut index = String::new();

    io::stdin()
        .read_line(&mut index)
        .expect("Failed to read line");

    let index: usize = index
        .trim()
        .parse()
        .expect("Index entered was not a number");

    let element = a[index];

    println!("The value of the element at index {index} is: {element}");
}
