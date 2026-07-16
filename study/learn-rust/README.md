# Rust Quick Reference

Personal cheat sheet while learning Rust. Covers setup → first program → core language features through ownership/borrowing.


- The Book: <https://doc.rust-lang.org/stable/book/index.html>
- The Book Local: `./book/book/index.html`
- Rust Documentations List: `rustup doc`

## Advanced
- The Rust Reference: <https://doc.rust-lang.org/stable/reference/introduction.html>
- Rust by Example: <https://doc.rust-lang.org/rust-by-example/>
- Rustlings (exercises): <https://github.com/rust-lang/rustlings>
- Std docs: <https://doc.rust-lang.org/std/>
---

## 1. Setup

### Install (Linux/macOS/WSL)

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
```

This installs:
- `rustc` — the compiler
- `cargo` — package manager + build tool (you'll use this 99% of the time)
- `rustup` — toolchain manager (update versions, switch nightly/stable)

### Verify

```bash
rustc --version
cargo --version
```

### Update later

```bash
rustup update
```

### Recommended editor setup

Install the language server (LSP) and lints as rustup components so they're tied to your active toolchain:

```bash
rustup component add rust-analyzer    # LSP — needed by VS Code, Neovim, etc.
rustup component add clippy           # extra lints, run via `cargo clippy`
rustup component add rustfmt          # formatter, run via `cargo fmt` (usually pre-installed)
```

- **VS Code:** install the `rust-analyzer` extension (NOT the older "Rust" one).
- **Neovim:** `rustaceanvim` (LazyVim's `lang.rust` extra) auto-attaches once the binary is on PATH.

#### Gotcha: "rust-analyzer not found" / infinite recursion

`~/.cargo/bin/rust-analyzer` is a **rustup proxy** (symlink to `rustup`), not the real binary. If you skip `rustup component add rust-analyzer`, the proxy can't dispatch — it falls back to `/usr/bin/rust-analyzer` (often another proxy), bounces back and forth, and exits with `error: infinite recursion detected`. Your editor reports the LSP crashed.

Fix is just to install the component:

```bash
rustup component add rust-analyzer
rustup which rust-analyzer    # should now print a real path under ~/.rustup/toolchains/...
rust-analyzer --version
```

Same trick (proxy → real binary) is used for `cargo`, `rustc`, `clippy`, `rustfmt`. They work out of the box because rustup installs them by default; `rust-analyzer` is opt-in.

---

## 2. Create & run a project

```bash
cargo new hello_world      # creates ./hello_world with Cargo.toml + src/main.rs
cd hello_world
cargo run                  # compile + run
cargo build                # compile (debug) → target/debug/
cargo build --release      # optimized build → target/release/
cargo check                # typecheck without producing a binary (fast feedback)
cargo test                 # run tests
cargo fmt                  # format
cargo clippy               # lint
```

### Project layout

```
hello_world/
├── Cargo.toml      # manifest: name, version, dependencies
├── Cargo.lock      # locked dep versions (commit for binaries, optional for libs)
└── src/
    └── main.rs     # entry point for a binary crate
```

For a library: `cargo new mylib --lib` → `src/lib.rs`.

### `.gitignore`

`cargo new` generates a minimal one (`/target`). A reasonable expanded version:

```gitignore
# Rust build artifacts
/target
**/target

# Backup files from older rustfmt
**/*.rs.bk

# Windows debug symbols
*.pdb

# Editor / IDE
.vscode/
.idea/
*.swp

# Env files
.env
.env.local
```

**`Cargo.lock` rule of thumb:**
- **Binary / app:** commit it (reproducible builds).
- **Library** (published to crates.io): don't commit it (downstream lockfiles win).
- Unsure? You're probably writing a binary → commit it.

### Adding dependencies

Edit `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
```
Or: `cargo add serde --features derive`.

---

## 3. First program

`src/main.rs`:
```rust
fn main() {
    println!("Hello, world!");
}
```

- `fn` declares a function. `main` is the entry point.
- `println!` is a **macro** (the `!`), not a function. Macros expand at compile time.
- Statements end with `;`. Expressions don't (more on this below).

---

## 4. Variables

```rust
let x = 5;              // immutable by default
let mut y = 10;         // mutable
y += 1;

const MAX: u32 = 100;   // const: compile-time, must annotate type, SCREAMING_CASE
```

### Shadowing

You can re-declare the same name. Different from mutation — type can change.

```rust
let n = "5";
let n: i32 = n.parse().unwrap();   // shadowed; n is now i32
```

---

## 5. Types

### Scalars
- Integers: `i8 i16 i32 i64 i128 isize` and unsigned `u8 ... usize`. Default: `i32`.
- Floats: `f32`, `f64`. Default: `f64`.
- Bool: `bool`.
- Char: `char` (4 bytes, Unicode scalar).

### Compound
- Tuple: `let t: (i32, f64, char) = (1, 2.0, 'a');` access via `t.0`.
- Array: `let a: [i32; 3] = [1, 2, 3];` fixed length, stack-allocated.
- Slice: `&a[0..2]` view into a contiguous sequence.

### Strings (two main types)
- `String` — heap-allocated, growable, owned.
- `&str` — immutable borrowed view ("string slice"). String literals are `&'static str`.

```rust
let s1: &str = "hello";              // literal
let s2: String = String::from("hi"); // owned
let s3: &str = &s2;                  // borrow String as &str
```

### Type inference

Rust infers most types. Annotate when ambiguous:
```rust
let n: u64 = 42;
let v: Vec<i32> = Vec::new();
```

---

## 6. Control flow

```rust
if x > 0 { ... } else if x == 0 { ... } else { ... }

// if is an expression
let parity = if n % 2 == 0 { "even" } else { "odd" };

loop { break; }                  // infinite loop, can return a value via break
while cond { ... }
for i in 0..10 { ... }           // range
for x in &vec { ... }            // iterator
```

`match` is the workhorse pattern-matcher:
```rust
match n {
    0 => println!("zero"),
    1 | 2 => println!("one or two"),
    3..=9 => println!("single digit"),
    _ => println!("other"),
}
```

---

## 7. Functions & expressions

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b   // no semicolon = expression = return value
}
```

- The last expression in a block is its value. `{ let x = 1; x + 2 }` evaluates to `3`.
- A trailing `;` turns it into a statement (returns `()`, the unit type).
- `return` works too but is uncommon for the final value.

---

## 8. Ownership (the Rust thing)

Three rules:
1. Each value has exactly one **owner**.
2. When the owner goes out of scope, the value is **dropped** (freed).
3. Assigning or passing a value **moves** it (for non-`Copy` types).

```rust
let s1 = String::from("hi");
let s2 = s1;            // s1 is MOVED into s2; s1 is no longer valid
// println!("{s1}");    // ERROR: borrow of moved value

let n1 = 5;
let n2 = n1;            // i32 is Copy → n1 still valid
println!("{n1} {n2}");
```

**`Copy` types** (cheap to duplicate, stack-only): all integers, floats, bools, chars, and tuples of `Copy` types. `String`, `Vec<T>`, etc. are NOT `Copy`.

### Clone

When you actually want a deep copy:
```rust
let s2 = s1.clone();    // both valid; allocates a new String
```

---

## 9. Borrowing (references)

Instead of moving, **borrow** with `&`:

```rust
fn len(s: &String) -> usize { s.len() }

let s = String::from("hello");
let n = len(&s);        // s is borrowed, not moved
println!("{s} has {n} chars");
```

### Mutable borrows

```rust
fn push_bang(s: &mut String) { s.push('!'); }

let mut s = String::from("hi");
push_bang(&mut s);
```

### The borrow rules (enforced at compile time)

At any given time, you can have **either**:
- Any number of immutable references (`&T`), **OR**
- Exactly one mutable reference (`&mut T`).

References must always be valid (no dangling).

```rust
let mut s = String::from("hi");
let r1 = &s;
let r2 = &s;            // OK: multiple immutable
// let r3 = &mut s;     // ERROR: can't mix with immutable refs
println!("{r1} {r2}");  // last use of r1, r2
let r3 = &mut s;        // OK now
r3.push('!');
```

### Slices

A slice is a borrowed view, written `&[T]` or `&str`:
```rust
let v = vec![1, 2, 3, 4];
let middle = &v[1..3];   // &[i32], borrow of v
```

---

## 10. Structs & enums

```rust
struct User {
    name: String,
    age: u32,
}

let u = User { name: String::from("Han"), age: 30 };
println!("{}", u.name);

// methods via impl
impl User {
    fn greet(&self) -> String {
        format!("hi, {}", self.name)
    }
}
```

Enums are sum types (much more powerful than C enums):
```rust
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle(r) => std::f64::consts::PI * r * r,
        Shape::Rect { w, h } => w * h,
    }
}
```

### `Option` and `Result` (no nulls, no exceptions)

```rust
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

Use them everywhere absence/failure is possible:
```rust
fn first_char(s: &str) -> Option<char> { s.chars().next() }

match first_char("hi") {
    Some(c) => println!("{c}"),
    None => println!("empty"),
}
```

The `?` operator propagates errors:
```rust
fn read() -> Result<String, std::io::Error> {
    let s = std::fs::read_to_string("file.txt")?;  // returns Err early on failure
    Ok(s)
}
```

---

## 11. Collections (the common ones)

```rust
let mut v: Vec<i32> = vec![1, 2, 3];
v.push(4);

use std::collections::HashMap;
let mut m = HashMap::new();
m.insert("a", 1);
```

Iterate:
```rust
for x in &v { println!("{x}"); }
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
```

---

## 12. Traits (interfaces)

```rust
trait Greet {
    fn hello(&self) -> String;
}

impl Greet for User {
    fn hello(&self) -> String { format!("hi {}", self.name) }
}

fn say<T: Greet>(t: &T) { println!("{}", t.hello()); }
```

Common derived traits: `#[derive(Debug, Clone, PartialEq)]` on structs/enums.

Print debug output: `println!("{:?}", value);` (or `{:#?}` pretty).

---

## 13. Lifetimes (briefly)

Most of the time the compiler infers them. When references in a return value depend on inputs, you annotate:

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

`'a` says: "the returned reference lives at least as long as both inputs." Don't worry about this until the compiler asks.

---

## 14. Workflow tips

### Checking for errors from the CLI

- `cargo check` — typecheck only, no codegen. Fastest feedback loop, run constantly.
- `cargo build` — full compile.
- `cargo clippy` — `cargo check` + extra lints. `cargo clippy -- -W clippy::pedantic` for stricter hints.
- Watch mode (auto re-check on save):
  ```bash
  cargo install cargo-watch
  cargo watch -x check
  ```

For a **single file** outside a cargo project:
```bash
rustc --edition 2021 main.rs                  # compiles to ./main
rustc --edition 2021 --emit=metadata main.rs  # check only, no binary produced
```

> Note: `rust-analyzer` is **not** a CLI checker — it's an LSP server that speaks JSON-RPC to your editor. Running `rust-analyzer main.rs` just hangs. Use `cargo check` for CLI feedback; let your editor handle the LSP side.

### Other tips

- Read compiler errors carefully; they're famously good and often suggest the fix.
- `dbg!(x)` prints `x` with file/line — handy `printf`-debugging macro.
- Tests live next to code:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn it_works() { assert_eq!(2 + 2, 4); }
  }
  ```

---
 ## 15. Unsafe Rust
```bash
rustup +nightly component add miri
cargo +nightly miri run
cargo +nightly miri test
```

- The Book: <https://doc.rust-lang.org/stable/book/index.html>
