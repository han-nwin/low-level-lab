// Boxes don’t have performance overhead, other than storing their data on the heap instead of on the stack. But they don’t have many extra capabilities either.
// You’ll use them most often in these situations:
//
//     When you have a type whose size can’t be known at compile time,
//         and you want to use a value of that type in a context that requires an exact size
//     When you have a large amount of data, and you want to transfer ownership but ensure
//         that the data won’t be copied when you do so
//     When you want to own a value, and you care only that it’s a type that implements
//        a particular trait rather than being of a specific type

// implemnt a cons list
// a data structure made up of nested pairs,
// e.g(1, (2, (3, Nil)))
enum List {
    Cons(i32, Box<List>),
    Nil,
}

// This line imports the `Cons` and `Nil` variants from the `List` enum,
// allowing us to use them directly in the code without needing to prefix them
// with `List::`. This improves code readability and makes it easier to work
// with the `List` data structure.
use crate::List::{Cons, Nil};

// Implement our own Box

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

use std::ops::Deref;

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0 // access field 0 of the struct
    }
}

fn main() {
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

    let x = 5;
    let y = Box::new(x);

    println!("x = {x}");
    println!("y = address of x = {:p}", y);
    println!("*y = {}", *y); // run *(y.deref()) behind the scenes

    let x = 5;
    let y = MyBox::new(x);

    assert_eq!(5, x);
    assert_eq!(5, *y); // run *(y.deref()) behind the scenes
    //

    // Deref coercion
    let m = MyBox::new(String::from("Rust"));

    hello(&m);
    // here &m turn &MyBox into &String, and from &String to &str
}

fn hello(name: &str) {
    println!("Hello, {name}!");
}

// For mutable reference. Coercion can also work
//
// From &T to &U when T: Deref<Target=U>
// From &mut T to &mut U when T: DerefMut<Target=U>
//
// From &mut T to &U when T: Deref<Target=U>
//
// // NOTE: Because of the borrowing rules,
// // if you have a mutable reference, that mutable reference must be
// // the only reference to that data (otherwise, the program wouldn’t compile).
// // Converting one mutable reference to one immutable reference will never break the borrowing rules.
// // Converting an immutable reference to a mutable reference would require that the initial immutable
// // reference is the only immutable reference to that data, but the borrowing rules don’t guarantee that.
// NOTE: Immutable references will never coerce to mutable references.
//
// THINK: Converting permission: mut = exclusive read and write, immutable = read only
