fn main() {
    // Type synonym and type alias

    // synonym
    type Kilometers = i32;

    let x: i32 = 5;
    let y: Kilometers = 5;

    println!("x + y = {}", x + y);

    // alias
    type Thunk = Box<dyn Fn() + Send + 'static>;

    let f: Thunk = Box::new(|| println!("hi"));

    fn takes_long_type(f: Thunk) {
        // --snip--
    }

    fn returns_long_type() -> Thunk {
        // --snip--
    }

    // Use it in Result
    use std::fmt;
    use std::io::Error;

    pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
        fn flush(&mut self) -> Result<(), Error>;

        fn write_all(&mut self, buf: &[u8]) -> Result<(), Error>;
        fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<(), Error>;
    } // <, Error> repeat alot

    // do this
    type Result<T> = std::result::Result<T, std::io::Error>;

    // Function that return never (!)
    // NOTE: Rust has a special type named ! that’s known in
    // type theory lingo as the empty type because it has no values. We prefer
    // to call it the never type because it stands in the place of the return
    // type when a function will never return. Here is an example:
    fn bar() -> ! {}
    // example
    let guess: u32 = match guess.trim().parse() {
        Ok(num) => num,
        Err(_) => continue,
    };
    // continue return never

    // == Dynamically sized type and Sized trait
    // If we want a type with unknown size at compile time
    // we can borrow the idea of &str -> str
    // aka use the metadata of the data as a type
    // have it behind a Box<> or Rc<>
    // what about generic? -> use Sized trait
    fn generic<T: ?Sized>(t: &T) { //NOTE: the ? relax the sized contraint at compile time
        // --snip--
    }
    //normal generic func is
    fn generic<T>(t: T) {
        // --snip--
    }
    // which equivalent with
    fn generic<T: Sized>(t: T) {
        // --snip--
    }
}
