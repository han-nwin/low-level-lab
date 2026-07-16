//Write a function that takes a string of words separated by
//spaces and returns the first word it finds in that string.
//If the function doesn't find a space in the string, the whole
//string must be one word, so the entire string should be returned.
fn main() {
    let mut s = String::from("hello world");

    let word = first_word(&s);
    println!("{word}");

    s.clear(); // this empties the String, making it equal to ""
    // word still has the value 5 here, but s no longer has any content that we
    // could meaningfully use with the value 5, so word is now totally invalid!

    //=== using iterator on the String can make it hard since s can be cleared an we lose context//
    // USE SLICES
    let s1 = String::from("hello world");

    let hello = &s1[0..5]; // 0->4
    let world = &s1[6..11]; // 6 -> 10

    let len = s1.len();
    // more variations
    let slice = &s1[0..2];
    let slice = &s1[..2];

    let slice = &s1[3..len];
    let slice = &s1[3..];

    let slice = &s1[0..len];
    let slice = &s1[..];

    let mut s = String::from("hello world");
    let word = first_word_2(&s);
    println!("{word}");

    // SLICE also works on array
    let a = [1, 2, 3, 4, 5];
    let slice = &a[0..2];
    assert_eq!(slice, &[1, 2]);
}

fn first_word(s: &String) -> usize {
    let bytes = s.as_bytes(); // turn to type u8 (unsigned 8-bit)

    for (i, &item) in bytes.iter().enumerate() {
        // turn char ' ' to u8
        if item == b' ' {
            return i;
        }
    }

    s.len()
}

// Find second word?
fn second_word(s: &String) -> (usize, usize) {
    (2, 2)
}

// Use slice
// use &str instead of &String so it can work on string literals and String values
fn first_word_2(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}
