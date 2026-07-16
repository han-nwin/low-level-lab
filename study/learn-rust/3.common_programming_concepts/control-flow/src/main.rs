fn main() {
    let number = 3;

    if number > 5 {
        println!("bigger")
    } else {
        println!("lower")
    }

    if number % 4 == 0 {
        println!("divisible by 4");
    } else if number % 2 == 0 {
        println!("divisible by 2");
    } else if number % 3 == 0 {
        println!("divisible by 3");
    } else {
        println!("what the hell?");
    }

    // Using if in a let satatement
    let result = if number % 2 == 0 { "even" } else { "odd" };
    println!("The number is {}", result);

    // Notes: The type is from the first block
    // let result = if number % 2 == 0 { 0 } else { "odd " }; // this WILL BREAK

    // LOOP
    // loop {
    //     println!("bro !!")
    // }

    // Returning value from loop
    let mut counter: i32 = 0;
    let cnt = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2; // return value after break
        }
    };
    println!("cnt = {cnt}");

    let mut count = 0;
    // 'counting_up is a loop label in Rust.
    // It lets you name a loop so break or continue can target a specific outer loop instead of just the nearest one.
    'counting_up: loop {
        println!("count = {count}");
        let mut remaining = 10;

        loop {
            println!("remaining = {remaining}");
            if remaining == 9 {
                break; // NOTES: break the inner loop 
            }
            if count == 2 {
                break 'counting_up; // NOTES: break the outer loop
            }
            remaining -= 1;
        }

        count += 1;
    }
    println!("End count = {count}");

    // while loop
    let mut number = 3;
    while number > 0 {
        println!("{number}");
        number -= 1;
    }
    println!("LIFTOFF!!");

    // Looping Through a Collection with for
    let a = [10, 20, 30, 40, 50];
    let mut index = 0;

    while index < 5 {
        println!("the value is: {}", a[index]);

        index += 1;
    }

    for element in a {
        println!("the value is: {element}");
    }

    // range and rev() to revert. exclude last number in the range
    for num in (1..4).rev() {
        println!("num = {num}");
    }
}
