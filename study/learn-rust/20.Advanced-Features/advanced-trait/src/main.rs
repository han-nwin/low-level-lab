// Define a trait with associated types
pub trait Iterator {
    type Item;
    type Bro;

    fn next(&mut self) -> Option<Self::Item>;
}
// NOTE: why not use Generic like
pub trait Iter<T> {
    fn next(&mut self) -> Option<T>;
}

//NOTE: Because we can have different implementations
// of next() on different types
// next() on a u32 may need to be different than next() on a String

#[allow(dead_code)]
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;
    type Bro = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 10 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

// === Operator overloading ===//
// NOTE: overload the operations and corresponding traits listed in std::ops by implementing the traits associated with the operator
use std::ops::Add;

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

// Add looks like this
// NOTE: Rhs = right hand side
// Rhs is the type of the value on the right-hand side of the `+` operator.
// trait Add<Rhs = Self> {
//     type Output;
//     fn add(self, rhs: Rhs) -> Self::Output;
// }

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// custom Rhs
struct Millimeters(u32);
struct Meters(u32);

// Adding Meters (rhs type) to Millimeters(the one we implementing Add on)
impl Add<Meters> for Millimeters {
    type Output = Millimeters;

    fn add(self, other: Meters) -> Millimeters {
        Millimeters(self.0 + (other.0 * 1000))
    }
}

// Ambiguous identical name
trait Pilot {
    fn fly(&self);
}

trait Wizard {
    fn fly(&self);
}

struct Human;

impl Pilot for Human {
    fn fly(&self) {
        println!("This is your captain speaking.");
    }
}

impl Wizard for Human {
    fn fly(&self) {
        println!("Up!");
    }
}

struct Airplane;

impl Pilot for Airplane {
    fn fly(&self) {
        println!("The airplane is taking off.");
    }
}

// NOTE: Trait object ==//
// A trait object lets different concrete types share one collection.
// Without `dyn Pilot`, a Vec must contain values of one concrete type.
fn fly_everything(pilots: &[Box<dyn Pilot>]) {
    for pilot in pilots {
        pilot.fly(); // dynamically calls the implementation for Human or Airplane
    }
}

impl Human {
    fn fly(&self) {
        println!("*waving arms furiously*");
    }
}

trait Animal {
    fn baby_name() -> String;
}

struct Dog;

impl Dog {
    fn baby_name() -> String {
        String::from("Spot")
    }
}

impl Animal for Dog {
    fn baby_name() -> String {
        String::from("puppy")
    }
}

// a trait definition that depends on another trait
// aka Supertrait
use std::fmt;

trait OutlinePrint: fmt::Display {
    fn outline_print(&self) {
        let output = self.to_string();
        let len = output.len();
        println!("{}", "*".repeat(len + 4));
        println!("*{}*", " ".repeat(len + 2));
        println!("* {output} *");
        println!("*{}*", " ".repeat(len + 2));
        println!("{}", "*".repeat(len + 4));
    }
}

impl OutlinePrint for Point {}

// have to implement Display for Point
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y) // return an fmt::Result
    }
}

//Implementing External Traits with the Newtype Pattern
// NOTE: we’re only allowed to implement a trait on a type if either the trait or the type, or both, are local to our crate.
// but we can get arround it with a wrapper
//

struct Wrapper(Vec<String>); // wrap the external type in our local struct

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

fn main() {
    assert_eq!(
        Point { x: 1, y: 0 } + Point { x: 2, y: 3 },
        Point { x: 3, y: 3 }
    );
    println!(
        "Point {{ x: 1, y: 0 }} + Point {{ x: 2, y: 3 }} = {:?}",
        Point { x: 1, y: 0 } + Point { x: 2, y: 3 }
    );

    let human = Human {};
    human.fly(); // what fly() is this????

    // More explicit syntax
    Pilot::fly(&human);
    Wizard::fly(&human);
    human.fly();

    // NOTE: Trait object usage
    // Human and Airplane are different types, but both implement Pilot.
    // `Box<dyn Pilot>` hides their concrete types behind the shared behavior.
    let pilots: Vec<Box<dyn Pilot>> = vec![Box::new(Human), Box::new(Airplane)];
    fly_everything(&pilots);

    println!("Dog struct baby_name print {}", Dog::baby_name());
    // NOTE: Use fully qualified syntax
    //<Type as Trait>::function(receiver_if_method, next_arg, ...);
    println!(
        "Animal trait baby_name print {}",
        <Dog as Animal>::baby_name()
    );

    let w = Wrapper(vec![String::from("hello"), String::from("world")]);
    let v = vec![String::from("hello"), String::from("world")];

    println!("wrapper display: = {w}");
    println!("normal vec display with :? only: {v:?}");
}
