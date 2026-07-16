fn main() {
    let mut v: Vec<i32> = Vec::new();

    // or use vec! macro
    let v2 = vec![1, 2, 3];

    v.push(2);
    v.push(3);
    v.push(5);

    println!("{:?}", v);
    println!("{:?}", v2);

    // Access elements
    let third: &i32 = &v[2];
    println!("The third element is {third}");

    let third: Option<&i32> = v.get(2);
    match third {
        Some(third) => println!("The third element is {third}"),
        None => println!("There is no third element."),
    }

    // let does_not_exist = &v[100]; // THIS WILL PANIC
    // let does_not_exist = v.get(100); // THIS RETURNS NONE
    //
    let mut v5 = vec![1, 2, 3, 4, 5];
    let first = &v5[0];
    // v.push(6); // mem realloc happens here
    // first variable (reference to the original vec) no longer valid

    println!("The first element is: {first}");

    // Iterating over values in vector
    let v3 = vec![100, 32, 57];
    for i in &v3 {
        println!("{i}");
    }

    // Updating
    let mut v = vec![100, 23, 44];
    for i in &mut v {
        *i += 50; // dereference the pointer
        println!("{i}")
    }
    // or
    for index in 0..v.len() {
        v[index] += 50;
        println!("{}", v[index]);
    }

    // Using enum to store multiple types
    #[derive(Debug)]
    enum SpreadsheetCell {
        Int(i32),
        Float(f64),
        Text(String),
    }

    let mut row = vec![
        SpreadsheetCell::Int(1),
        SpreadsheetCell::Text(String::from("hello")),
        SpreadsheetCell::Float(3.32134),
    ];

    for i in &row {
        println!("{:?}", *i);
    }
} // All vec inside this {} goes out of scrope after this
