struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

fn main() {
    let c = CustomSmartPointer {
        data: String::from("my stuff"),
    };
    let d = CustomSmartPointer {
        data: String::from("other stuff"),
    };

    println!("CustomSmartPointers created");
    // NOTE: Rust doesn’t let us call drop explicitly,
    // because Rust would still automatically call drop on the value at the end of main.
    // This would cause a double free error because Rust would be trying to clean up the same value twice.
    // We can’t disable the automatic insertion of drop when a value goes out of scope,
    // and we can’t call the drop method explicitly. So, if we need to force a value to be cleaned up early,
    // we use the std::mem::drop function.

    // force drop
    drop(c); //std::mem::drop not the CustomerSmartPointer::drop().

    println!("CustomSmartPointer dropped before the end of main");
} // drop() wll be called when data is out of scope
