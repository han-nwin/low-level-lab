**Packages**: A Cargo feature that lets you build, test, and share crates
**Crates**: A tree of modules that produces a library or executable
**Modules** and use: Let you control the organization, scope, and privacy of paths
**Paths**: A way of naming an item, such as a struct, function, or module

# Rust 7.1: Packages and Crates

                    package
        "one Cargo project / one bundle"
                 Cargo.toml
                      |
      +---------------+---------------+
      |                               |

src/main.rs src/lib.rs
binary crate library crate
executable app reusable code
| |
cargo run used by other code
|
creates a program

## Important rules:

Package
= a Cargo project
= has Cargo.toml

Crate
= smallest unit Rust compiler compiles

Binary crate
= has main()
= makes executable program

Library crate
= no main()
= provides reusable functions/types

One package can contain:

- 0 or 1 library crate
- any number of binary crates
- at least 1 crate total
