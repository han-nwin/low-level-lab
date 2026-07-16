fn main() {
    // 2. Turn string literal to string type
    let data = "initial dataaaa";
    let s2 = data.to_string();
    println!("{s2}");

    // The method also works on a literal directly:
    let s = "initial contents".to_string();
    println!("{s}");

    // 1. Create an empty string
    let mut s1 = String::new();
    // append
    s1.push_str(" brotherrrr");
    println!("{s1}");
    // append 1 char
    s1.push('h');
    println!("{s1}");

    // Concatenating
    let sa = String::from("Hello, ");
    let sb = String::from("world!");
    let sc = sa + &sb; // note sa has been moved here and can no longer be used
    // the + operator use this method fn add(self, s: &str) -> String  under the hood
    println!("{sc}");

    let s1 = String::from("tic");
    let s2 = String::from("tac");
    let s3 = String::from("toe");

    // let s = s1 + "-" + &s2 + "-" + &s3; // TOO long
    //instead
    let _ = format!("{s1}-{s2}-{s3}");
    println!("{s}");

    //Internal structure
    // A String is a wrapper over a Vec<u8>
    let hello = String::from("Hola"); //length 4 here
    let hello = String::from("Здравствуйте"); // this is 24 in lenght, since each Unicode scalar
    // value takes 2 bytes
    // let answer = &hello[0];
    //
    // Slicing a string
    let hello = "Здравствуйте";
    let s5 = &hello[0..6];
    println!("{s5}");

    // Iterating Over Strings
    for c in hello.chars() {
        println!("{c}");
    }
    for c in hello.bytes() {
        println!("{c}");
    }
}
