#[derive(Debug)]
struct Rectangle {
    width: u64,
    height: u64,
}

// Method
// fn read(&self)       // Rust auto-borrows immutably
// fn change(&mut self) // Rust auto-borrows mutably
// fn take(self)        // Rust moves ownership
impl Rectangle {
    fn area(&self) -> u64 {
        self.width * self.height
    }

    fn can_hold(&self, other_rec: &Rectangle) -> bool {
        self.area() > other_rec.area()
    }

    // no need self function
    fn square(size: u64) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };
    let rect2 = Rectangle {
        width: 10,
        height: 40,
    };
    let rect3 = Rectangle {
        width: 60,
        height: 45,
    };

    let square = Rectangle::square(10);
    println!("square: {square:?}");

    println!("Can rect1 hold rect2? {}", rect1.can_hold(&rect2));
    println!("Can rect1 hold rect3? {}", rect1.can_hold(&rect3));
}
