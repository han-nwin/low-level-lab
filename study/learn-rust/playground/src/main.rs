fn main() {
    let st = String::from("yoo");
    let pt: &str = &st; //&st first creates a &String, then Rust coerces it into &str. aka fat
    //pointer
    let pt2: &String = &st;

    println!("The string: {st}");

    // Actual heap address
    println!("heap address of first element: {:p}", st.as_ptr());
    // Wrapper value on the stack
    println!("wrapper value on the stack: {:p}", pt); // fat pointer
    // Pointer to the wrapper on the stack
    println!("pointer to the wrapper value on the stack: {:p}", pt2);
    // Address of the Pointer to the wrapper on the stack
    println!(
        "address of the pointer to the wrapper value on the stack: {:p}",
        &pt2
    );
    // Address on the stack of the wrapper
    println!("address of the wrapper on the stack {:p}", &pt);

    println!("------Create a heap copy with clone-----");
    let copy = st.clone();
    let copy_pt = &copy;

    println!("The string: {copy}");
    println!("heap address: {:p}", copy.as_ptr());
    println!("wrapper value on the stack: {:p}", copy_pt);
    println!("address of the wrapper on the stack {:p}", &copy_pt);

    // &String -> String { ptr, len, cap } -> heap bytes. We own

    // &&str   -> &str { ptr, len } -------> heap bytes. We borrow
}
