// As such, Rust does not have nulls,
// but it does have an enum that can encode the concept of a value being present or absent.
// This enum is Option<T>, and it is defined by the standard library as follows:

// enum Option<T> {
//     None,
//     Some(T),
// }

fn main() {
    let some_number = Some(5);
    let some_char = Some('e');

    let absent_number: Option<i32> = None;

    let x: i8 = 5;
    let y: Option<i8> = Some(5);

    // let sum = x + y; // this will throw compiler error

    //convert y to i8 instead of Option<i8>
    // match is like a switch statement
    let sum = match y {
        Some(val) => val + x,
        None => x,
    };
    println!("sum: {}", sum);
}
