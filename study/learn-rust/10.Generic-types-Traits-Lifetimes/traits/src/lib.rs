// Using Trait Bounds to Conditionally Implement Methods
use std::fmt::Display;

struct Pair<T> {
    x: T,
    y: T,
}

impl<T> Pair<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("The largest member is x = {}", self.x);
        } else {
            println!("The largest member is y = {}", self.y);
        }
    }
}
//We can also conditionally implement a trait for any type that
//implements another trait. Implementations of a trait on any
//type that satisfies the trait bounds are called blanket implementations
// and are used extensively in the Rust standard library.
// 
// For example, the standard library implements the ToString trait on
// any type that implements the Display trait. The impl block in the
// standard library looks similar to this code:
impl<T: Display> ToString for T {
    // --snip--
}

// Because the standard library has this blanket implementation,
// we can call the to_string method defined by the ToString trait
// on any type that implements the Display trait. 
let s = 3.to_string();
