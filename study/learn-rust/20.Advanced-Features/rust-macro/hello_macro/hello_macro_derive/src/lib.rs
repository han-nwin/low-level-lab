use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate.
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation.
    impl_hello_macro(&ast)
}

// DeriveInput {
//     // --snip--
//
//     ident: Ident {
//         ident: "Pancakes",
//         span: #0 bytes(95..103)
//     },
//     data: Struct(
//         DataStruct {
//             struct_token: Struct,
//             fields: Unit,
//             semi_token: Some(
//                 Semi
//             )
//         }
//     )
// }
fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generated = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    generated.into()
}
