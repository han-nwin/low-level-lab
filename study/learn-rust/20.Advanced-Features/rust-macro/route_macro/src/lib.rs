//! Defines the `#[route(METHOD, "path")]` attribute-like macro.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, Ident, ItemFn, LitStr, Pat, ReturnType, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Marks a one-argument function as a route handler and generates a dispatcher.
///
/// `#[route(GET, "/users/{id}")]` on `get_user(id: u64)` generates a
/// `get_user_dispatch(method, path)` function. The dispatcher matches the
/// method and path, parses the `{id}` segment as `u64`, and calls `get_user`.
#[proc_macro_attribute]
pub fn route(attribute: TokenStream, item: TokenStream) -> TokenStream {
    // Attribute-like macros receive two streams. Parse the first as `METHOD,
    // "path"` and the second as the complete annotated function.
    let RouteArgs {
        method,
        _comma: _,
        path,
    } = parse_macro_input!(attribute as RouteArgs);
    let function = parse_macro_input!(item as ItemFn);

    let method_text = method.to_string();
    if !matches!(
        method_text.as_str(),
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
    ) {
        return syn::Error::new_spanned(method, "expected GET, POST, PUT, PATCH, or DELETE")
            .to_compile_error()
            .into();
    }

    if let Some(async_token) = function.sig.asyncness {
        return syn::Error::new_spanned(
            async_token,
            "#[route] currently supports synchronous handlers",
        )
        .to_compile_error()
        .into();
    }

    // This learning example supports one `{parameter}` in the route path.
    let path_text = path.value();
    let Some(open_brace) = path_text.find('{') else {
        return syn::Error::new_spanned(
            path,
            "route path must contain one parameter, such as {id}",
        )
        .to_compile_error()
        .into();
    };
    let Some(relative_close_brace) = path_text[open_brace + 1..].find('}') else {
        return syn::Error::new_spanned(path, "route path has an unclosed parameter")
            .to_compile_error()
            .into();
    };
    let close_brace = open_brace + 1 + relative_close_brace;

    let parameter_name = &path_text[open_brace + 1..close_brace];
    let prefix = &path_text[..open_brace];
    let suffix = &path_text[close_brace + 1..];
    if parameter_name.is_empty() || prefix.contains(['{', '}']) || suffix.contains(['{', '}']) {
        return syn::Error::new_spanned(
            path,
            "route path must contain exactly one named parameter",
        )
        .to_compile_error()
        .into();
    }

    if function.sig.inputs.len() != 1 {
        return syn::Error::new_spanned(
            &function.sig.inputs,
            "route handler must have exactly one argument matching the path parameter",
        )
        .to_compile_error()
        .into();
    }
    let argument = function
        .sig
        .inputs
        .first()
        .expect("the length check guarantees one argument");
    let FnArg::Typed(argument) = argument else {
        return syn::Error::new_spanned(argument, "route handler cannot take self")
            .to_compile_error()
            .into();
    };
    let Pat::Ident(argument_pattern) = argument.pat.as_ref() else {
        return syn::Error::new_spanned(&argument.pat, "route argument must be a simple name")
            .to_compile_error()
            .into();
    };
    if argument_pattern.ident != parameter_name {
        return syn::Error::new_spanned(
            &argument_pattern.ident,
            format!("argument must be named `{parameter_name}` to match the route path"),
        )
        .to_compile_error()
        .into();
    }

    let handler_name = &function.sig.ident;
    let visibility = &function.vis;
    let argument_type = &argument.ty;
    let dispatcher_name = format_ident!("{handler_name}_dispatch"); // i.e: get_user_dispatch
    let method_literal = LitStr::new(&method_text, method.span());
    let prefix_literal = LitStr::new(prefix, path.span());
    let suffix_literal = LitStr::new(suffix, path.span());
    let return_type = match &function.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, output) => quote! { #output },
    };

    quote! {
        // Preserve the user's handler exactly as it was written.
        #function

        // Match a simulated request and invoke the handler when it fits.
        #visibility fn #dispatcher_name(
            request_method: &str,
            request_path: &str,
        ) -> ::std::option::Option<#return_type> {
            if request_method != #method_literal {
                return ::std::option::Option::None;
            }

            let parameter = request_path
                .strip_prefix(#prefix_literal)?
                .strip_suffix(#suffix_literal)?;
            if parameter.is_empty() || parameter.contains('/') {
                return ::std::option::Option::None;
            }

            let #argument_pattern: #argument_type = parameter.parse().ok()?;
            ::std::option::Option::Some(#handler_name(#argument_pattern))
        }
    }
    .into()
}

/// Parsed representation of the tokens inside `#[route(...)]`.
struct RouteArgs {
    method: Ident,
    _comma: Token![,],
    path: LitStr,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            method: input.parse()?,
            _comma: input.parse()?,
            path: input.parse()?,
        })
    }
}
