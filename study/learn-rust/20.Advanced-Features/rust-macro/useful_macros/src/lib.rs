//! Procedural macros that can be reused by other Rust crates.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Measures a function call and prints its elapsed time when the function exits.
///
/// # Example
///
/// ```ignore
/// use useful_macros::timed;
///
/// #[timed]
/// fn process_file() {
///     // Expensive work...
/// }
/// ```
///
/// The generated guard uses `Drop`, so timing is reported for normal returns,
/// explicit early returns, and unwinding panics. The macro preserves the
/// function's arguments, visibility, attributes, and return value.
#[proc_macro_attribute]
pub fn timed(attribute: TokenStream, item: TokenStream) -> TokenStream {
    if !attribute.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[timed] does not accept arguments",
        )
        .to_compile_error()
        .into();
    }

    // Turn the raw tokens into a function-shaped syntax tree. If `#[timed]` is
    // put on something other than a function, syn emits a useful compiler error.
    let mut function = parse_macro_input!(item as ItemFn);
    let function_name = function.sig.ident.clone();
    let original_body = function.block;

    // Wrap the original body in a new block containing a timer guard. Rust
    // drops the guard whenever the function exits, which triggers the report.
    function.block = Box::new(syn::parse_quote!({
        struct __TimedGuard {
            function_name: &'static str,
            started_at: ::std::time::Instant,
        }

        impl ::std::ops::Drop for __TimedGuard {
            fn drop(&mut self) {
                ::std::println!(
                    "[timed] {} took {:?}",
                    self.function_name,
                    self.started_at.elapsed(),
                );
            }
        }

        let _timed_guard = __TimedGuard {
            function_name: ::std::stringify!(#function_name),
            started_at: ::std::time::Instant::now(),
        };

        #original_body
    }));

    // Convert the modified syntax tree back into tokens for rustc to compile.
    quote!(#function).into()
}
