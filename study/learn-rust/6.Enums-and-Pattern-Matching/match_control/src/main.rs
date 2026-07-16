#[derive(Debug)] // so we can inspect the state in a minute
enum UsState {
    Alabama,
    NewYork,
}

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState),
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            println!("State quarter from {:?}!", state);
            25
        }
    }
}
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}
// can use x.map(|i| i + 1) to get the same result

fn main() {
    let quarter = Coin::Quarter(UsState::NewYork);
    println!("value_in_cents: {:?}", value_in_cents(quarter));

    let five = Some(5);
    let six = plus_one(five);
    let none = plus_one(None);
    println!("plus_one: {:?}", six);
    println!("plus_one: {:?}", none);

    let dice_roll = 9;
    match dice_roll {
        3 => add_fancy_hat(),
        7 => remove_fancy_hat(),
        _ => (), // trick to exhause the match arms by just ignoring them
    }
}

fn add_fancy_hat() {}
fn remove_fancy_hat() {}
