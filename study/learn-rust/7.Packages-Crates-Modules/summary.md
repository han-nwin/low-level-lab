# Chapter 7 Summary: Packages, Crates, and Modules

Chapter 7 explains how Rust organizes growing codebases.

- A `package` is a Cargo project described by `Cargo.toml`.
- A package contains one or more `crates`.
- A crate is the smallest compilation unit in Rust.
- A crate can be a `binary crate` with `main`, or a `library crate` that exposes reusable code.

Rust organizes code inside a crate with the module system:

- `mod` declares modules and builds a module tree.
- Paths name items in that tree.
- Absolute paths usually start with `crate`.
- Relative paths usually start from the current module, `self`, or `super`.

Privacy is enforced by default:

- Items inside modules are private unless marked `pub`.
- Making a module `pub` does not automatically make its contents public.
- Public structs can still have private fields.
- Public enums expose all of their variants automatically.

The `use` keyword shortens long paths inside a scope:

- For functions, idiomatic Rust usually imports the parent module, then calls `module::function()`.
- For structs and enums, idiomatic Rust usually imports the full item path.
- `as` can rename imports to avoid conflicts.
- `pub use` re-exports an item so callers can use a cleaner public API.

Rust also lets you split modules into separate files:

- `mod front_of_house;` loads `src/front_of_house.rs`.
- A child module like `hosting` can live in `src/front_of_house/hosting.rs`.
- `mod` declares the module once; other code uses paths, not repeated includes.

The main design idea of the chapter is this: use packages, crates, modules, paths, and visibility rules to keep code easy to navigate while exposing only the API you want other code to depend on.

## Modules Cheat Sheet

- Start from the crate root: When compiling a crate, the compiler first looks in the crate root file (usually **src/lib.rs** for a library crate and **src/main.rs** for a binary crate) for code to compile.

- Declaring modules: In the crate root file, you can declare new modules; say you declare a “garden” module with mod garden;. The compiler will look for the module’s code in these places:
  Inline, within curly brackets that replace the semicolon following mod garden
  In the file src/garden.rs
  In the file src/garden/mod.rs

- Declaring submodules: In any file other than the crate root, you can declare submodules. For example, you might declare mod vegetables; in src/garden.rs. The compiler will look for the submodule’s code within the directory named for the parent module in these places:
  Inline, directly following mod vegetables, within curly brackets instead of the semicolon
  In the file src/garden/vegetables.rs
  In the file src/garden/vegetables/mod.rs

- Paths to code in modules: Once a module is part of your crate, you can refer to code in that module from anywhere else in that same crate, as long as the privacy rules allow, using the path to the code. For example, an Asparagus type in the garden vegetables module would be found at crate::garden::vegetables::Asparagus.

- Private vs. public: Code within a module is private from its parent modules by default. To make a module public, declare it with pub mod instead of mod. To make items within a public module public as well, use pub before their declarations.

- The use keyword: Within a scope, the use keyword creates shortcuts to items to reduce repetition of long paths. In any scope that can refer to crate::garden::vegetables::Asparagus, you can create a shortcut with use crate::garden::vegetables::Asparagus;, and from then on you only need to write Asparagus to make use of that type in the scope.

## Best practice of binary and library crate

> The module tree should be defined in src/lib.rs. Then, any public items can be used in the binary crate by starting paths with the name of the package. The binary crate becomes a user of the library crate just like a completely external crate would use the library crate: It can only use the public API. _This helps you design a good API; not only are you the author, but you’re also a client!_
