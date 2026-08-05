Rust uses **attributes** to give extra instructions to the compiler or to a macro.

Think of them like:

* Java annotations: `@Override`
* C/C++ compiler attributes: `[[nodiscard]]`
* Python decorators: `@dataclass`

But Rust has two attribute forms:

```rust
#![something]   // inner attribute
#[something]    // outer attribute
```

# 1. `#![...]` — inner attributes

An inner attribute applies to the **thing containing it**.

At the top of `main.rs` or `lib.rs`, the containing thing is the entire crate:

```rust
#![no_std]
#![no_main]
```

Meaning:

```text
#![no_std]
└─ applies to the entire crate

#![no_main]
└─ applies to the entire crate
```

This is common in embedded Rust:

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}
```

Here:

* `#![no_std]`: do not link Rust’s standard library
* `#![no_main]`: do not use the normal desktop `main` startup process

The `!` inside `#![...]` does **not** mean “not.” It marks this as an **inner attribute**.

## Inner attributes inside modules

An inner attribute can also apply to a module:

```rust
mod hardware {
    #![allow(dead_code)]

    fn unused_function() {}
}
```

This means:

```text
allow dead_code warnings inside the entire hardware module
```

---

# 2. `#[...]` — outer attributes

An outer attribute applies to the **next item**.

```rust
#[allow(dead_code)]
fn unused_function() {}
```

The attribute applies only to that function.

```rust
#[derive(Debug)]
struct User {
    name: String,
}
```

The attribute applies to the `User` struct.

```rust
#[repr(C)]
struct RegisterBlock {
    control: u32,
    status: u32,
}
```

The attribute applies to the struct.

A useful visual:

```rust
#[some_attribute]
struct MyStruct {}
//     └────────── the attribute modifies this item
```

---

# 3. `#[derive(...)]`

`derive` automatically generates trait implementations.

Without `derive`:

```rust
struct Point {
    x: i32,
    y: i32,
}
```

You cannot print it with `{:?}`:

```rust
let point = Point { x: 1, y: 2 };

println!("{point:?}"); // error
```

Add `Debug`:

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}
```

Rust effectively generates something resembling:

```rust
impl Debug for Point {
    // Generated implementation
}
```

You do not need to manually write it.

## Multiple derives

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
```

Each name is a trait implementation to generate.

Common derives:

| Derive       | What it gives you                       |
| ------------ | --------------------------------------- |
| `Debug`      | Print with `{:?}`                       |
| `Clone`      | Explicitly duplicate with `.clone()`    |
| `Copy`       | Duplicate automatically by copying bits |
| `PartialEq`  | Compare with `==` and `!=`              |
| `Eq`         | Full equality guarantee                 |
| `PartialOrd` | Compare with `<`, `>`, etc.             |
| `Ord`        | Full ordering                           |
| `Default`    | Create a default value                  |
| `Hash`       | Use as a hash map/set key               |

Example:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let a = Point::default();
    let b = a; // Point is Copy

    println!("{a:?}");
    println!("{}", a == b);
}
```

`Default` generates an implementation based on the defaults of the fields. For integers, the default is `0`.

---

# 4. General attribute syntax

Attributes commonly have three shapes.

## Path only

```rust
#[test]
#[inline]
#[cold]
```

This is simply an attribute name.

## List of arguments

```rust
#[derive(Debug, Clone)]
#[allow(dead_code, unused_variables)]
#[cfg(target_arch = "arm")]
```

The attribute receives arguments inside parentheses.

## Name-value syntax

```rust
#[path = "other_file.rs"]
#[doc = "This is documentation"]
```

The attribute receives a key-value-like value.

So the general forms are:

```rust
#[name]

#[name(arguments)]

#[name = "value"]
```

---

# 5. Attributes do not normally run at runtime

Most attributes are handled during compilation.

For example:

```rust
#[allow(dead_code)]
fn unused() {}
```

This does not execute any code.

It tells the compiler:

```text
Do not produce a dead_code warning for this function.
```

Likewise:

```rust
#[derive(Debug)]
struct Data {}
```

The compiler or macro generates code before your program runs.

---

# 6. Important built-in attributes

## `#[allow(...)]`

Suppresses compiler warnings:

```rust
#[allow(dead_code)]
fn experimental_function() {}
```

Other levels include:

```rust
#[warn(dead_code)]
#[deny(dead_code)]
#[forbid(unsafe_code)]
```

Their meanings:

```text
allow    Ignore the lint
warn     Produce a warning
deny     Turn it into a compile error
forbid   Turn it into an error and prevent overriding it later
```

Example applying to the whole crate:

```rust
#![deny(unsafe_code)]
```

Example applying to one function:

```rust
#[allow(unused_variables)]
fn example() {
    let value = 42;
}
```

---

## `#[cfg(...)]`

Conditionally compiles code.

```rust
#[cfg(target_arch = "arm")]
fn platform_name() {
    println!("ARM");
}

#[cfg(target_arch = "x86_64")]
fn platform_name() {
    println!("x86-64");
}
```

Only the matching function is included in the compiled program.

Common conditions:

```rust
#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_arch = "arm")]
#[cfg(debug_assertions)]
#[cfg(test)]
#[cfg(feature = "logging")]
```

You can combine conditions:

```rust
#[cfg(all(target_arch = "arm", target_os = "none"))]
fn embedded_only() {}
```

```rust
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_like() {}
```

```rust
#[cfg(not(feature = "logging"))]
fn no_logging() {}
```

---

## `#[cfg_attr(...)]`

Conditionally applies another attribute.

```rust
#![cfg_attr(not(test), no_std)]
```

Meaning:

```text
When we are not compiling tests, apply #![no_std].
```

General form:

```rust
#[cfg_attr(condition, attribute_to_apply)]
```

---

## `#[repr(...)]`

Controls data layout.

This is especially important in embedded systems and C interoperability.

```rust
#[repr(C)]
struct RegisterBlock {
    control: u32,
    status: u32,
}
```

`repr(C)` asks Rust to lay out the fields using C-compatible layout rules.

For enums:

```rust
#[repr(u8)]
enum Command {
    Start = 1,
    Stop = 2,
}
```

Now the enum representation is based on `u8`.

Common forms:

```rust
#[repr(C)]
#[repr(u8)]
#[repr(u16)]
#[repr(u32)]
#[repr(transparent)]
#[repr(align(64))]
#[repr(packed)]
```

Example from your atomic alignment lab:

```rust
#[repr(align(64))]
struct CacheAligned<T>(T);
```

This forces the struct to have 64-byte alignment.

Be careful with:

```rust
#[repr(packed)]
```

Packed fields may be unaligned, which can make normal references unsafe or invalid.

---

## `#[inline]`

Suggests that the compiler insert a function’s instructions directly into the caller:

```rust
#[inline]
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

Variants:

```rust
#[inline]
#[inline(always)]
#[inline(never)]
```

This is mostly a compiler optimization hint. The compiler may already inline functions without it.

---

## `#[test]`

Marks a function as a test:

```rust
#[test]
fn addition_works() {
    assert_eq!(2 + 2, 4);
}
```

When you run:

```bash
cargo test
```

Rust finds functions marked with `#[test]` and runs them.

---

## `#[must_use]`

Warns when the result is ignored:

```rust
#[must_use]
fn calculate_value() -> u32 {
    42
}

fn main() {
    calculate_value(); // warning: unused return value
}
```

This is why ignoring a `Result` often causes a warning:

```rust
std::fs::write("output.txt", "hello");
```

Many important types such as `Result` are marked `#[must_use]`.

---

## `#[deprecated]`

Marks something as outdated:

```rust
#[deprecated(note = "Use new_function instead")]
fn old_function() {}

fn new_function() {}
```

Calling `old_function()` produces a warning.

---

## `#[panic_handler]`

Used in `no_std` programs to define what happens during panic:

```rust
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

The compiler expects exactly one panic handler in a final embedded binary.

---

# 7. Attribute macros such as `#[entry]`

This is where attributes become more powerful.

In embedded Rust:

```rust
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}
```

`#[entry]` is not a normal built-in compiler attribute. It is an **attribute procedural macro** supplied by `cortex-m-rt`.

It receives your function and transforms or validates it.

Conceptually:

```text
Your source:

#[entry]
fn main() -> ! {
    loop {}
}

Macro processing:

1. Verify the function has the correct signature
2. Connect it to the reset/startup machinery
3. Export the required symbol
4. Generate supporting code
```

Other embedded frameworks have similar macros:

```rust
#[interrupt]
fn TIMER0() {
    // Interrupt handler
}
```

```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Async embedded entry point
}
```

The full path can appear inside an attribute:

```rust
#[some_crate::some_macro]
fn example() {}
```

---

# 8. Three kinds of procedural macros

Rust has three major procedural macro forms.

## Derive macro

```rust
#[derive(Debug)]
struct User {}
```

It generates trait implementations.

Custom crates can provide derives:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}
```

Here, `Serialize` and `Deserialize` usually come from Serde.

---

## Attribute macro

```rust
#[tokio::main]
async fn main() {}
```

It modifies the item placed underneath it.

Another example:

```rust
#[route("/users")]
fn users() {}
```

The exact behavior depends on the crate defining the macro.

---

## Function-like macro

```rust
println!("hello");
vec![1, 2, 3];
format!("value = {}", value);
```

These use:

```rust
macro_name!(...)
```

This is different syntax from an attribute:

```rust
#[attribute]
item
```

Compare:

```rust
#[derive(Debug)]     // attribute macro / derive attribute
struct Data {}

println!("hello");   // function-like macro
```

---

# 9. Why do macros use `!`?

Function-like macros use a bang:

```rust
println!()
vec![]
format!()
```

It tells you:

```text
This is a macro invocation, not a normal function call.
```

Macros can accept token syntax that normal functions cannot.

For example:

```rust
vec![1, 2, 3]
```

A normal function call must use parentheses:

```rust
some_function(1, 2, 3)
```

Macros can use any matching delimiter:

```rust
my_macro!(...)
my_macro![...]
my_macro! {...}
```

The `!` in:

```rust
#![no_std]
```

has a different grammatical purpose. It marks an **inner attribute**.

So:

```text
println!()   ! means macro invocation
#![no_std]   ! means inner attribute
```

Same character, different Rust syntax rules.

---

# 10. Documentation comments are attributes too

This:

```rust
/// Adds two values.
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

is essentially syntax sugar for:

```rust
#[doc = "Adds two values."]
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

An inner documentation comment:

```rust
//! This crate controls an RP2350.
```

is similar to:

```rust
#![doc = "This crate controls an RP2350."]
```

Therefore:

```text
/// documents the next item
//! documents the containing module or crate
```

This mirrors:

```text
#[...]  outer attribute
#![...] inner attribute
```

---

# 11. Attributes can be stacked

An item can have several attributes:

```rust
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[must_use]
struct Measurement {
    value: u32,
}
```

Each attribute has a separate job:

```text
derive      Generate trait implementations
repr(C)     Control memory layout
must_use    Warn when a returned value is ignored
```

This is equivalent formatting:

```rust
#[derive(
    Debug,
    Clone,
    Copy,
)]
#[repr(C)]
struct Measurement {
    value: u32,
}
```

---

# 12. Attribute placement matters

This applies to the struct:

```rust
#[derive(Debug)]
struct Device {
    enabled: bool,
}
```

This applies to one field:

```rust
struct Device {
    #[allow(dead_code)]
    enabled: bool,
}
```

This applies to one function:

```rust
#[inline]
fn enable() {}
```

This applies to the whole crate:

```rust
#![no_std]
```

The basic rule is:

```text
#[...]  looks forward to the next item
#![...] looks outward to its containing item
```

---

# 13. Your typical embedded Rust file

Here is how to read a realistic embedded program:

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum LedState {
    Off = 0,
    On = 1,
}

#[entry]
fn main() -> ! {
    let state = LedState::On;

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

Line by line:

```rust
#![no_std]
```

Apply `no_std` to the entire crate.

```rust
#![no_main]
```

Apply `no_main` to the entire crate.

```rust
#[derive(Debug, Clone, Copy)]
```

Generate three trait implementations for `LedState`.

```rust
#[repr(u8)]
```

Give the enum a `u8` representation.

```rust
#[entry]
```

Let `cortex-m-rt` convert this function into the embedded entry point.

```rust
#[panic_handler]
```

Tell the compiler this function handles panics.

---

# 14. One compact mental model

When you see:

```rust
#![something]
```

Read it as:

> Configure the thing I am currently inside.

When you see:

```rust
#[something]
```

Read it as:

> Configure or transform the next item.

When you see:

```rust
#[derive(Trait)]
```

Read it as:

> Generate an implementation of this trait for the next type.

When you see:

```rust
something!(...)
```

Read it as:

> Invoke a function-like macro.

So these four examples mean:

```rust
#![no_std]          // configure the entire crate
#[repr(C)]          // configure the next struct
#[derive(Debug)]    // generate Debug for the next type
println!("hello");  // invoke the println macro
```
