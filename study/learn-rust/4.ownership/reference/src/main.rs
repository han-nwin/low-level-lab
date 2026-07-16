// A reference is a a pointer pointing to the address on
// the stack and read its' value (can either be on stack or heap)
// Stack:
//
// s
// ┌────────────────────┐
// │ ptr ─────────────┐ │
// │ len = 5          │ │
// │ cap = 5          │ │
// └────────────────────┘ │
//                        │
// Heap:                  │
// ┌────────────────────┐ │
// │ h e l l o          │◄┘
// └────────────────────┘
//
// r
// ┌────────────────────┐
// │ address of s       │
// └────────────────────┘
// At any given time, you can have either
// one mutable reference or any number of immutable references.
// References must always be valid.
fn main() {
    let s1 = String::from("hello");

    let len = calculate_length(&s1);

    println!("The length of '{s1}' is {len}.");

    // let s = String::from("hello");
    // change1(&s);

    let mut s = String::from("hello");
    change(&mut s);
    println!("{s}");

    //NOTES: Rust only allow 1 mutable reference alive at the same time
    // aka. use 1 first then create new one
    // let r1 = &mut s;
    // let r2 = &mut s;
    // println!("{r1}, {r2}");
    // => this will break
    // DO this instead
    {
        let r1 = &mut s;
    } // r1 goes out of scope here, so we can make a new reference with no problems.

    let r2 = &mut s;

    // This is also NOT allowed. read - write pointers are hard to synced
    let r1 = &s; // no problem
    let r2 = &s; // no problem
    let r3 = &mut s; // BIG PROBLEM

    // a reference's scope starts from where it is introduced and continues
    // through the last time that reference is used. For instance, this code
    // will compile because the last usage of the immutable references is in the
    // println!, before the mutable reference is introduced:
    let r1 = &s; // no problem
    let r2 = &s; // no problem
    println!("{r1} and {r2}");
    // Variables r1 and r2 will not be used after this point.

    let r3 = &mut s; // no problem
    println!("{r3}");
    // These scopes don't overlap, so this code is allowed:

    let reference_to_nothing = dangle();
}

fn calculate_length(s: &String) -> usize {
    // s is a reference to a String
    s.len()
} // Here, s goes out of scope. But because s does not have ownership of what
// it refers to, the String is not dropped.

// This will throw error if we tryna modify from a referene
// fn change1(some_string: &String) {
//     some_string.push_str(", world");
// }

// Make it a mutable reference then we can modify
fn change(some_string: &mut String) {
    some_string.push_str(", world");
}

fn dangle() -> &String {
    // dangle returns a reference to a String

    let s = String::from("hello"); // s is a new String

    &s // we return a reference to the String, s
} // Here, s goes out of scope and is dropped, so its memory goes away.
// Danger!
