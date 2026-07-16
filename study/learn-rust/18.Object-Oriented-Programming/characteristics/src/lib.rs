// Struc and enum are objects
// NOTE: Encapsulation
// Aka private -> hide implementation detail
pub struct AveragedCollection {
    list: Vec<i32>,
    average: f64,
}

impl AveragedCollection {
    pub fn add(&mut self, value: i32) {
        self.list.push(value);
        self.update_average();
    }

    pub fn remove(&mut self) -> Option<i32> {
        let result = self.list.pop();
        match result {
            Some(value) => {
                self.update_average();
                Some(value)
            }
            None => None,
        }
    }

    pub fn average(&self) -> f64 {
        self.average
    }

    fn update_average(&mut self) {
        let total: i32 = self.list.iter().sum();
        self.average = total as f64 / self.list.len() as f64;
    }
}

// NOTE: INHERITANCE
// is a mechanism whereby an object can inherit elements
// from another object’s definition, thus gaining the parent
// object’s data and behavior without you having to define them again.
//
// Rust by default doesn't have that
// but we can have similar behavior using trait

pub trait Summary {
    fn summarize(&self) -> String;
}
// Anything that implement Summary must have summarize()

// NOTE: POLYMOPHISM
// Refer to a child class modifying what it inherits from a parent class.
// Again in Rust, use trait for this

// === USING TRAIT OBJECT TO Abtract over shared behavior ===//
// NOTE:
// We’ll create a library crate called gui that contains the structure of a
// GUI library. This crate might include some types for people to use, such
// as Button or TextField. In addition, gui users will want to create their
// own types that can be drawn: For instance, one programmer might add an
// Image, and another might add a SelectBox

pub mod gui {

    pub trait Draw {
        fn draw(&self);
    }

    // A screen contain components, where component can be any tpe that has Draw trait
    pub struct Screen {
        pub components: Vec<Box<dyn Draw>>, // trait objects
    }

    impl Screen {
        pub fn run(&self) {
            for component in self.components.iter() {
                component.draw();
            }
        }
    }
    // NOTE: -> This is using trait object

    // Whilst the above can have multiple types in the Vec, if we do it
    // like the way below it only allow 1 type
    // pub struct Screen<T: Draw> {
    //     pub components: Vec<T>,
    // }
    //
    // impl<T> Screen<T>
    // where
    //     T: Draw, // trait bound
    // {
    //     pub fn run(&self) {
    //         for component in self.components.iter() {
    //             component.draw();
    //         }
    //     }
    // }
    //NOTE : -> This is using trait bound

    pub struct Button {
        pub width: u32,
        pub height: u32,
        pub label: String,
    }

    impl Draw for Button {
        fn draw(&self) {
            let width = self.width as usize;
            let height = self.height as usize;
            let label = self.label.clone();

            if width < 2 || height < 2 {
                println!("Width and height must be at least 2.");
                return;
            }

            println!("┌{}┐", "─".repeat(width - 2));

            for _ in 0..height / 2 - 1 {
                println!("│{}│", " ".repeat(width - 2));
            }
            if label.len().is_multiple_of(2) {
                println!(
                    "│{}{label}{}│",
                    " ".repeat((width - label.len()) / 2 - 1),
                    " ".repeat((width - label.len()) / 2 - 1)
                );
            } else {
                println!(
                    "│{}{label}{}│",
                    " ".repeat((width - label.len()) / 2),
                    " ".repeat((width - label.len()) / 2 - 1)
                );
            }

            for _ in 0..height / 2 - 1 {
                println!("│{}│", " ".repeat(width - 2));
            }

            println!("└{}┘", "─".repeat(width - 2));
        }
    }

    // NOTE: Extend existing type
    // We can't do this in main because Draw and String are not local to main crate
    // one of them must be local, in lib.rs Draw is local so this is good
    impl Draw for String {
        fn draw(&self) {
            println!("{}", self);
        }
    }
}
