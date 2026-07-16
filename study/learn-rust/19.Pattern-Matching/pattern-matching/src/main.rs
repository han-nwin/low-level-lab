fn main() {
    //match VALUE {
    //     PATTERN => EXPRESSION,
    //     PATTERN => EXPRESSION,
    //     PATTERN => EXPRESSION,
    // }
    let x = Some(2);
    #[allow(clippy::manual_map)]
    let val = match x {
        None => None,
        Some(i) => Some(i + 1),
    };
    println!("{}", val.unwrap());

    // let PATTERN = EXPRESSION;
    let (x, y, z) = (1, 2, 3);

    // Conditional if let
    let favorite_color: Option<&str> = None;
    let is_tuesday = false;
    let age: Result<u8, _> = "34".parse();

    if let Some(color) = favorite_color {
        println!("Using your favorite color, {color}, as the background");
    } else if is_tuesday {
        println!("Tuesday is green day!");
    } else if let Ok(age) = age {
        if age > 30 {
            println!("Using purple as the background color");
        } else {
            println!("Using orange as the background color");
        }
    } else {
        println!("Using blue as the background color");
    }

    // While let pattern
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for val in [1, 2, 3] {
            tx.send(val).unwrap();
        }
    });

    while let Ok(value) = rx.recv() {
        println!("{value}");
    }

    // for loop pattern
    let v = ['a', 'b', 'c'];

    for (index, value) in v.iter().enumerate() {
        println!("{value} is at index {index}");
    }

    // refutable patern
    let Some(x) = some_option_value else {
        return;
    };

    let point = (3, 5);
    print_coordinates(&point);
}
// funtion params
fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("Current location: ({x}, {y})");
}
