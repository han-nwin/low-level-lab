fn main() {
    let x = 1;

    match x {
        1 => println!("one"),
        2 => println!("two"),
        3 => println!("three"),
        _ => println!("anything"),
    }

    //============//
    let x = Some(5);
    let y = 10;
    match x {
        Some(50) => println!("Got 50"),
        Some(y) => println!("Matched, y = {y}"), // this y is not the same as the previous
        // (different scope)
        _ => println!("Default case: x = {x:?}"),
    }
    println!("At the end: x = {x:?}, y = {y}");

    // another version
    match x {
        Some(50) => println!("Got 50"),
        Some(n) if n == y => println!("Matched, n = {n}"),
        _ => println!("Default case, x = {x:?}"),
    }
    println!("at the end: x = {x:?}, y = {y}");
    //============//

    // matching multiple pattern
    let x = 1;
    match x {
        1 | 2 => println!("one or two"),
        3 => println!("three"),
        _ => println!("something else"),
    }

    // matching range values
    let x = 5;
    match x {
        1..=5 => println!("on through five (included)"),
        _ => println!("something else"),
    }
    // matching range of char values
    let x = 'c';
    match x {
        'a'..='j' => println!("early ASCII letter"),
        'k'..='z' => println!("late ASCII letter"),
        _ => println!("something else"),
    }

    // Destructuring to Break Apart Values
    struct Point {
        x: i32,
        y: i32,
    }

    let point = Point { x: 2, y: 10 };

    let Point { x: a, y: b } = point; // = is a copy here 
    assert_eq!(a, 2);
    assert_eq!(b, 10);

    // short hand
    let Point { x, y } = point;
    assert_eq!(x, 2);
    assert_eq!(y, 10);

    let p = Point { x: 0, y: 7 };
    match p {
        // match both x =0 y=0
        Point { x: 0, y: 0 } => println!("p is at the origin"),

        // match any x but y = 0
        Point { x, y: 0 } => println!("p is on the x axis at {x}"),

        // match any y but x = 0
        Point { x: 0, y } => println!("p is onn the y axis at {y}"),

        // match any x and y
        Point { x, y } => println!("p is on neither axis: ({x}, {y})"),
    }

    // Matching Enum
    #[allow(dead_code)]
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        ChangeColor(i32, i32, i32),
    }
    let msg = Message::ChangeColor(1, 2, 3);

    match msg {
        Message::Quit => println!("the Quit variance has no data to destruct"),
        Message::Move { x, y } => println!("Move in the x = {x} direction and y ={y} direction"),
        Message::Write(s) => println!("Write {s}"),
        Message::ChangeColor(r, g, b) => println!("Change the color to r = {r}, g = {g}, b = {b}"),
    }

    // Nested struct and enum
    #[allow(dead_code)]
    enum Color {
        Rgb(i32, i32, i32),
        Hsv(i32, i32, i32),
    }

    #[allow(dead_code)]
    enum Message2 {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        ChangeColor(Color),
    }

    let msg = Message2::ChangeColor(Color::Hsv(0, 160, 255));

    match msg {
        Message2::ChangeColor(Color::Rgb(r, g, b)) => {
            println!("Change color to red {r}, green {g}, and blue {b}");
        }
        Message2::ChangeColor(Color::Hsv(h, s, v)) => {
            println!("Change color to hue {h}, saturation {s}, value {v}");
        }
        _ => (),
    }

    // Struct and tuple
    #[allow(unused_variables)]
    let ((feet, inches), Point { x, y }) = ((3, 10), Point { x: 3, y: -10 });

    // _ in function
    fn foo(_: i32, y: i32) {
        println!("This code only uses the y parameter: {y}");
    }
    foo(3, 4);

    // the user should not be allowed to overwrite an existing customization of a setting but
    // can unset the setting and give it a value if it is currently unset.
    let mut setting_value = Some(5);
    let new_setting_value = Some(10);
    // if any of the above is None then it'll use the later hand
    match (setting_value, new_setting_value) {
        (Some(_), Some(_)) => println!("Cannot overwrite an existing customized value"),
        _ => setting_value = new_setting_value,
    }
    println!("setting is {:?}", setting_value);

    // Ignoring some parts of a tuple
    let numbers = (2, 4, 8, 16, 32);
    #[allow(clippy::match_single_binding)]
    match numbers {
        (first, _, third, _, fifth) => {
            println!("Some numbers: {first}, {third}, {fifth}");
        }
    }

    // unused var
    let _x = 5;
    // can use it for something like
    let str = Some(String::from("Yooo"));
    #[allow(clippy::redundant_pattern_matching)]
    if let Some(_) = str {
        // we don't need to get the value
        println!("Found a string");
    }

    // Remaining Parts of a Value with ..
    #[allow(dead_code)]
    struct Point3 {
        x: i32,
        y: i32,
        z: i32,
    }

    let origin = Point3 { x: 0, y: 0, z: 0 };

    let Point3 { x, .. } = origin;
    println!("x = {x}");

    let numbers = (2, 4, 8, 16, 32);
    let (first, .., last) = numbers;
    println!("first = {first}, last = {last}");

    //Adding Conditionals with Match Guards
    //The downside of this additional expressiveness is that the compiler doesn’t try
    //to check for exhaustiveness when match guard expressions are involved.
    let num = Some(4);
    match num {
        Some(x) if x % 2 == 0 => println!("x is even"),
        Some(_) => println!("x is odd"),
        None => println!("x is None"),
    }

    let x = 4;
    let y = false;
    match x {
        4..=6 if y => println!("yes"),
        _ => println!("no"),
    }

    // USING @ BINDINGS
    // The at operator @ lets us create a variable that holds a value at the same time we’re testing that value for a pattern match

    enum Message3 {
        Hello { id: i32 },
    }
    let msg = Message3::Hello { id: 5 };
    match msg {
        //id @ before the range 3..=7, we’re capturing whatever value matched the range in a variable named id
        Message3::Hello { id: id @ 3..=7 } => println!("found an id in range {id}"),
        //doesn’t have a variable that contains the actual value of the id field.
        Message3::Hello { id: 10..=12 } => {
            println!("Found an id in another range")
        }
        Message3::Hello { id } => println!("Found some other id: {id}"),
    }
}
