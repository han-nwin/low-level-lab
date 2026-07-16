fn main() {
    // Passing function to function
    // NOTE: Function pointer
    fn add_one(x: i32) -> i32 {
        x + 1
    }

    //Unlike closures, fn is a type rather than a trait, so we
    //specify fn as the parameter type directly rather than declaring a generic type parameter with
    //one of the Fn traits as a trait bound.
    fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {
        f(arg) + f(arg)
    }

    let answer = do_twice(add_one, 5);
    println!("The answer is: {answer}");

    // replace closure with a named function

    let list_of_numbers = vec![1, 2, 3];
    let list_of_strings: Vec<String> = list_of_numbers.iter().map(|i| i.to_string()).collect();

    let list_of_numbers = vec![1, 2, 3];
    let list_of_strings: Vec<String> = list_of_numbers.iter().map(ToString::to_string).collect();

    //the name of each enum variant that we define also becomes an initializer function.
    //We can use these initializer functions as function pointers
    enum Status {
        Value(u32),
        Stop,
    }
    let list_of_statuses: Vec<Status> = (0u32..20).map(Status::Value).collect();

    // Returning Closure
    //Closures are represented by traits,
    //which means you can’t return closures directly.
    //In most cases where you might want to return a trait,
    fn returns_closure() -> impl Fn(i32) -> i32 {
        |x| x + 1
    };

    fn returns_initialized_closure(init: i32) -> impl Fn(i32) -> i32 {
        move |x| x + init
    };
    // each closure is also its own distinct type.
    // If you need to work with multiple functions that have the same signature
    // but different implementations, you will need to use a trait object for them.
    // NOTE: use a trait object instead
    fn returns_closure() -> Box<dyn Fn(i32) -> i32> {
        Box::new(|x| x + 1)
    }

    fn returns_initialized_closure(init: i32) -> Box<dyn Fn(i32) -> i32> {
        Box::new(move |x| x + init)
    }
}
