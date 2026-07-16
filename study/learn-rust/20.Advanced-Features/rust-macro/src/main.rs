// NOTE:
// Macros
// We’ve used macros like println! throughout this book,
// but we haven’t fully explored what a macro is and how it works.
//
// The term macro refers to a family of features in Rust—declarative
// macros with macro_rules! and three kinds of procedural macros:
//     Custom #[derive] macros that specify code added with the
//         derive attribute used on structs and enums
//     Attribute-like macros that define custom attributes usable on any item
//     Function-like macros that look like function calls but operate on the
//         tokens specified as their argument

// macros are a way of writing code that writes other code, which is known as metaprogramming

//NOTE: DECLARETIVE MACRO
// Example, vec! macro
#[macro_export]
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

// NOTE: PROCEDURAL MACROS FOR GENERATING CODE FROM ATTRIBUTES
// Procedural macros are Rust functions that run at compile time. They receive
// Rust source code as a TokenStream, inspect or transform it, and return the
// TokenStream that the compiler should compile. They must be defined in a
// separate crate whose type is `proc-macro`; see useful-macros/src/lib.rs.
//
// Conceptually, a TokenStream is a cheaply clonable, recursive Vec<TokenTree>:
//
// enum TokenTree {
//     Ident(Ident),       // Names and keywords: `fn`, `add`, `i32`
//     Punct(Punct),       // Punctuation: `+`, `:`, `-`, `>`
//     Literal(Literal),   // Values: `1`, `"hello"`, `'x'`
//     Group(Group),       // A nested TokenStream inside `()`, `{}`, or `[]`
// }
//
// For example, `fn add(x: i32) -> i32 { x + 1 }` looks roughly like:
//
// TokenStream [
//     Ident("fn"), Ident("add"),
//     Group(Parenthesis, [Ident("x"), Punct(':'), Ident("i32")]),
//     Punct('-'), Punct('>'), Ident("i32"),
//     Group(Brace, [Ident("x"), Punct('+'), Literal("1")]),
// ]
//
// Each token also has a Span describing where it came from. Spans let compiler
// errors point back to the relevant source code and affect name resolution.
// A TokenStream is not an AST: crates such as `syn` parse these raw tokens into
// structures like functions and structs, while `quote` turns structures back
// into tokens.
//
// PRACTICAL EXAMPLE: MEASURE HOW LONG A FUNCTION TAKES
//
// 1. Cargo builds the `useful-macros` proc-macro crate first.
// 2. `use` brings its exported `timed` attribute into this crate.
// 3. The compiler encounters `#[timed]` above `load_user`.
// 4. It removes the attribute and calls the macro during compilation:
//
//    timed(
//        attribute = TokenStream [],
//        item = TokenStream for `fn load_user(user_id: u64) -> String { ... }`,
//    )
//
// 5. `syn` parses the raw item tokens into an ItemFn (a function-shaped AST).
// 6. The macro adds a small timer guard to the start of the function body.
//    `quote` converts the modified function back into a TokenStream.
// 7. The compiler compiles code equivalent to:
//
//    fn load_user(user_id: u64) -> String {
//        let timer = Timer::start("load_user");
//        // original function body runs here
//        // timer's Drop implementation prints the elapsed time on every exit
//    }
//
// 8. The macro runs only while compiling. The generated timer runs at runtime.
//
// This is useful for quickly instrumenting database calls, file processing,
// startup work, or other functions without repeating timing boilerplate.
use useful_macros::timed;

#[timed]
fn load_user(user_id: u64) -> String {
    // Pretend this is a database or network request.
    std::thread::sleep(std::time::Duration::from_millis(20));
    format!("user-{user_id}")
}

// NOTE: CUSTOM DERIVE MACROS
use hello_macro::HelloMacro; // trait
use hello_macro_derive::HelloMacro; //derive macro

#[derive(HelloMacro)]
struct Pancakes;

// NOTE: ATTRIBUTE-LIKE MACROS
//
// Attribute-like macros look like custom Rust attributes and receive two
// TokenStreams:
//
// 1. `GET, "/users/{id}"` -- tokens inside the attribute's parentheses.
// 2. The complete `fn get_user(...) { ... }` item below the attribute.
//
// Our `route` macro validates the HTTP method and generates a dispatcher at
// compile time. The generated code is roughly:
//
// fn get_user_dispatch(method: &str, path: &str) -> Option<String> {
//     // Match GET and /users/{id}, parse id as u64, then call get_user(id).
// }
//
// This is real request matching and handler dispatch, but uses strings instead
// of a network server so the macro behavior stays easy to see.
use route_macro::route;

#[route(GET, "/users/{id}")]
fn get_user(id: u64) -> String {
    format!("User #{id}")
}

fn main() {
    let v: Vec<u32> = vec![1, 2, 3];
    println!("Declarative vec! result: {v:?}");

    let user = load_user(42);
    println!("Loaded {user}");

    Pancakes::hello_macro();

    // `get_user_dispatch` does not appear in our source; #[route] generated it.
    for (method, path) in [
        ("GET", "/users/42"),
        ("POST", "/users/42"),
        ("GET", "/not-users/42"),
    ] {
        match get_user_dispatch(method, path) {
            Some(response) => println!("{method} {path} -> 200: {response}"),
            None => println!("{method} {path} -> no matching route"),
        }
    }
}
