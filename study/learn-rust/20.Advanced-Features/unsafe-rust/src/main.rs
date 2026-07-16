fn main() {
    //NOTE: Dereferencing a Raw Pointer
    // Are allowed to ignore the borrowing rules by having both immutable and
    // mutable pointers or multiple mutable pointers to the same location
    // Aren’t guaranteed to point to valid memory
    // Are allowed to be null
    // Don’t implement any automatic cleanup

    let mut num = 5;

    let r1 = &raw const num; // creates a *const i32 immutable raw pointer
    let r2 = &raw mut num; // creates a *mut i32 mutable raw pointer
    unsafe {
        println!("r1 is {}", *r1);
        println!("r2 is {}", *r2);
    }

    // create a raw pointer from an arbitrary address
    let address = 0x012345usize;
    let r = address as *const i32;

    // Calling an unsafe function
    unsafe fn dangerous() {}
    unsafe {
        dangerous();
    }

    // Creating safe abstraction for unsafe code
    let mut v = vec![1, 2, 3, 4, 5, 6];
    let r = &mut v[..];

    let (a, b) = r.split_at_mut(3); //NOTE: Study this function

    assert_eq!(a, &mut [1, 2, 3]);
    assert_eq!(b, &mut [4, 5, 6]);

    fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
        use std::slice;
        let len = values.len();
        let ptr = values.as_mut_ptr();

        assert!(mid <= len);

        // Compiler don't know these 2 are non overlapping mutatble reference
        // (&mut values[..mid], &mut values[mid..])

        // Use unsafe code instead
        unsafe {
            (
                slice::from_raw_parts_mut(ptr, mid),
                slice::from_raw_parts_mut(ptr.add(mid), len - mid), // ptr.add -> move it forward
            )
        }
    }

    // Calling code from other languages
    // NOTE
    unsafe extern "C" {
        //NOTE: this abs() is from #include <stdlib.h>
        // check `man abs`
        fn abs(input: i32) -> i32;
    }

    unsafe {
        println!("Absolute value of -3 according to C: {}", abs(-3));
    }

    // Calling our own .c code
    // compile it. Make sure we have cc dependency
    cc::Build::new().file("src/helper.c").compile("helper");
    // declare it
    unsafe extern "C" {
        // ffi = Foreign Function Interface
        // we use C int type here not i32
        fn double_number(value: std::ffi::c_int) -> std::ffi::c_int;
    }
    //use it
    unsafe { println!("Double number from helper.c: {}", double_number(10)) }

    // Calling Rust code from other languages
    #[unsafe(no_mangle)]
    pub extern "C" fn call_from_c() {
        println!("Just called a Rust function from C!");
    } // accessible from C code, after it’s compiled to a shared library and linked from C

    // Accessing or Modifying a Mutable Static Variable
    //'static lifetime, which means the Rust compiler can figure out the lifetime and we aren’t required to annotate it explicitly
    //NOTE: A subtle difference between constants and immutable static variables is that values
    //in a static variable have a fixed address in memory. Using the value will
    //always access the same data. Constants, on the other hand, are allowed
    //to duplicate their data whenever they’re used. Another difference is that static
    //variables can be mutable. Accessing and modifying mutable static variables is unsafe.
    static mut COUNTER: u32 = 0;

    /// SAFETY: Calling this from more than a single thread at a time is undefined
    /// behavior, so you *must* guarantee you only call it from a single thread at
    /// a time.
    unsafe fn add_to_count(inc: u32) {
        unsafe {
            COUNTER += inc;
        }
    }
    unsafe {
        // SAFETY: This is only called from a single thread in `main`.
        add_to_count(3);
        println!("COUNTER: {}", *(&raw const COUNTER)); // create a raw pointer to COUNTER then
        // deref
    }

    // Unsafe trait
    unsafe trait Foo {
        // methods go here
    }

    unsafe impl Foo for i32 {
        // method implementations go here
    }

    // Accessing fields of a union
    //The key property of unions is that all fields of a union share common storage
    //  Unlike a struct, a union holds only one field’s value at a time.
    union MyUnion {
        f1: u32,
        f2: f32,
    }
    let new_union = MyUnion { f2: 200.5 }; // NOTE: only use f2 here let's see if we try access f1

    unsafe fn get_f1(u: &MyUnion) -> u32 {
        unsafe { u.f1 }
    }
    unsafe fn get_f2(u: &MyUnion) -> f32 {
        unsafe { u.f2 }
    }
    let val_1 = unsafe { get_f1(&new_union) };
    let val_2 = unsafe { get_f2(&new_union) };
    println!("val_1: {}, val_2: {}", val_1, val_2); //val_1: 1128824832, val_2: 200.5
    //
    //NOTE: Use miri to compile unsafe Rust
    // rustup +nightly component add miri
    // cargo +nightly miri run
    // cargo +nightly miri test
}
