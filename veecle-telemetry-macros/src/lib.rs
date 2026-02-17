// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.
// Copyright 2025 Veecle GmbH.
//
// This file has been modified from the original TiKV implementation.

//! An attribute macro designed to eliminate boilerplate code for [`veecle_telemetry`](https://crates.io/crates/veecle_telemetry).

#![recursion_limit = "256"]
#![cfg_attr(not(feature = "enable"), allow(dead_code))]
#![cfg_attr(not(feature = "enable"), allow(unreachable_code))]

use std::collections::HashMap;

use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::*;

struct Arguments {
    name: Option<LitStr>,
    short_name: bool,
    properties: Vec<Property>,
    veecle_telemetry_crate: Option<syn::Path>,
    span: Span,
}

struct Property {
    key: LitStr,
    value: Lit,
    span: Span,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: LitStr = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Lit = input.parse()?;

        // For some reason, `join` fails in doc macros.
        let span = key.span().join(value.span()).unwrap_or_else(|| key.span());
        Ok(Property { key, value, span })
    }
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut short_name = false;
        let mut properties = Vec::<Property>::new();
        let mut veecle_telemetry_crate = None;
        let mut seen = HashMap::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            if seen.contains_key(&ident.to_string()) {
                return Err(Error::new(ident.span(), "duplicate argument"));
            }
            seen.insert(ident.to_string(), ());
            input.parse::<Token![=]>()?;
            match ident.to_string().as_str() {
                "name" => {
                    let parsed_name: LitStr = input.parse()?;
                    name = Some(parsed_name);
                }
                "short_name" => {
                    let parsed_short_name: LitBool = input.parse()?;
                    short_name = parsed_short_name.value;
                }
                "properties" => {
                    let content;
                    let _brace_token = braced!(content in input);
                    let property_list = content.parse_terminated(Property::parse, Token![,])?;
                    for property in property_list {
                        if properties
                            .iter()
                            .any(|existing| existing.key == property.key)
                        {
                            return Err(Error::new(Span::call_site(), "duplicate property key"));
                        }
                        properties.push(property);
                    }
                }
                "crate" => {
                    let crate_path: syn::Path = input.parse()?;
                    veecle_telemetry_crate = Some(crate_path);
                }
                _ => return Err(Error::new(Span::call_site(), "unexpected identifier")),
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }

        Ok(Arguments {
            name,
            short_name,
            properties,
            veecle_telemetry_crate,
            span: input.span(),
        })
    }
}

/// An attribute macro designed to eliminate boilerplate code.
///
/// This macro automatically creates a span for the annotated function. The span name defaults to
/// the function name but can be customized by passing a string literal as an argument using the
/// `name` parameter.
///
/// The `#[trace]` attribute requires a local parent context to function correctly. Ensure that
/// the function annotated with `#[trace]` is called within __a local context of a `Span`__, which
/// is established by invoking the `Span::set_local_parent()` method.
///
/// ## Arguments
///
/// * `name` - The name of the span. Defaults to the full path of the function.
/// * `short_name` - Whether to use the function name without path as the span name. Defaults to `false`.
/// * `properties` - A list of key-value pairs to be added as properties to the span. The value can be a format string,
///   where the function arguments are accessible. Defaults to `{}`.
///
/// # Examples
///
/// ```
/// use veecle_telemetry::instrument;
///
/// #[veecle_telemetry::instrument]
/// fn simple() {
///     // ...
/// }
///
/// #[veecle_telemetry::instrument(short_name = true)]
/// async fn simple_async() {
///     // ...
/// }
///
/// #[veecle_telemetry::instrument(properties = { "k1": "v1", "a": 2 })]
/// async fn properties(a: u64) {
///     // ...
/// }
/// ```
///
/// The code snippets above will be expanded to:
///
/// ```
/// # extern crate alloc;
/// # use veecle_telemetry::Span;
/// # use veecle_telemetry::protocol::transient::KeyValue;
///
/// fn simple() {
///     let __guard__ = Span::new("example::simple", &[]).entered();
///     // ...
/// }
///
/// async fn simple_async() {
///     veecle_telemetry::future::FutureExt::with_span(
///         async move {
///             // ...
///         },
///         veecle_telemetry::Span::new("simple_async", &[]),
///     )
///     .await
/// }
///
/// async fn properties(a: u64) {
///     veecle_telemetry::future::FutureExt::with_span(
///         async move {
///             // ...
///         },
///         veecle_telemetry::Span::new("example::properties", &[
///             KeyValue::new("k1", "v1"),
///             KeyValue::new("a", 2),
///         ]),
///     )
///     .await
/// }
/// ```
#[proc_macro_attribute]
pub fn instrument(
    arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    #[cfg(not(feature = "enable"))]
    {
        let _ = parse_macro_input!(arguments as Arguments);
        return item;
    }

    let arguments = parse_macro_input!(arguments as Arguments);
    let input = parse_macro_input!(item as ItemFn);

    let veecle_telemetry_crate = arguments
        .veecle_telemetry_crate
        .clone()
        .map(Ok)
        .unwrap_or_else(veecle_telemetry_path);
    let veecle_telemetry_crate = match veecle_telemetry_crate {
        Ok(path) => path,
        Err(error) => return error.to_compile_error().into(),
    };

    let function_name = &input.sig.ident;

    let block = match generate_block(
        function_name,
        &input.block,
        input.sig.asyncness.is_some(),
        &arguments,
        &veecle_telemetry_crate,
    ) {
        Ok(block) => block,
        Err(error) => return error.to_compile_error().into(),
    };

    let ItemFn {
        attrs, vis, sig, ..
    } = input;

    // Interpolate `#sig` so that all signature tokens (including `fn`, parens, etc.) preserve
    // their original source spans for correct LLVM coverage of the first line.
    // Interpolate `#block` (a `Block` with the original brace token) so that the body braces
    // retain the user's source spans; braces from `quote!()` carry a macro-expansion
    // `SyntaxContext` that causes Rust's coverage instrumentor to skip the function entirely.
    quote!(
        #(#attrs) *
        #vis #sig
        #block
    )
    .into()
}

/// Returns the span name expression.
///
/// Uses `type_name_of_val(&|| ())` via a declarative macro helper to capture the
/// full nesting path of the function.
/// The closure must come from a declarative macro (not directly from the proc macro) because
/// proc-macro-generated closure tokens cause rustc's coverage instrumentor to drop the function
/// signature's coverage region.
fn generate_name(
    function_name: &Ident,
    arguments: &Arguments,
    async_context: bool,
    veecle_telemetry_crate: &syn::Path,
) -> syn::Result<proc_macro2::TokenStream> {
    let span = function_name.span();
    if let Some(name) = &arguments.name {
        if name.value().is_empty() {
            return Err(Error::new(span, "`name` can not be empty"));
        }

        if arguments.short_name {
            return Err(Error::new(
                Span::call_site(),
                "`name` and `short_name` can not be used together",
            ));
        }

        Ok(quote_spanned!(span=> #name))
    } else if arguments.short_name {
        let function_name = function_name.to_string();
        Ok(quote_spanned!(span=> #function_name))
    } else {
        // Route through a declarative macro so the closure tokens originate from the
        // compiler's own expansion, preserving LLVM coverage on function signatures.
        Ok(quote_spanned!(span=>
            #veecle_telemetry_crate::__private_function_path!(#async_context)
        ))
    }
}

fn generate_properties(
    arguments: &Arguments,
    veecle_telemetry_crate: &syn::Path,
) -> proc_macro2::TokenStream {
    if arguments.properties.is_empty() {
        return quote::quote!(&[]);
    }

    let span = arguments.span;
    let properties = arguments
        .properties
        .iter()
        .map(|Property { key, value, span }| {
            quote_spanned!(*span=>
                #veecle_telemetry_crate::protocol::transient::KeyValue::new(#key, #value)
            )
        });
    let properties = Punctuated::<_, Token![,]>::from_iter(properties);
    quote_spanned!(span=>
        &[ #properties ]
    )
}

/// Generates the instrumented function body as a [`Block`] reusing the original brace tokens.
///
/// For async functions, wraps the body in `veecle_telemetry::future::FutureExt::with_span`.
/// For sync functions, enters a span guard before the body.
///
/// The returned [`Block`] preserves the original brace token spans from the user's source.
/// This is critical for LLVM coverage: braces produced by `quote!()` carry a macro-expansion
/// `SyntaxContext` that causes `rustc`'s coverage instrumentor to skip the function entirely.
/// Original statements are spliced via `#(#stmts)*` to preserve their source spans for
/// correct LLVM coverage mapping (see <https://github.com/veecle/veecle-os/issues/262>).
fn generate_block(
    func_name: &Ident,
    block: &Block,
    async_context: bool,
    arguments: &Arguments,
    veecle_telemetry_crate: &syn::Path,
) -> syn::Result<Block> {
    let name = generate_name(func_name, arguments, async_context, veecle_telemetry_crate)?;
    let properties = generate_properties(arguments, veecle_telemetry_crate);
    let stmts = &block.stmts;
    let span = func_name.span();

    let wrapper: Block = if async_context {
        // Build `async move { ... }` manually so the block's brace tokens carry the original
        // source spans.  The `async move` block is a separate closure/generator from `rustc`'s
        // perspective, so its body span is subject to the same `eq_ctxt` coverage filter as the
        // outer function body.
        let async_block = Expr::Async(ExprAsync {
            attrs: Vec::new(),
            async_token: token::Async { span },
            capture: Some(token::Move { span }),
            block: Block {
                brace_token: block.brace_token,
                stmts: block.stmts.clone(),
            },
        });

        syn::parse2(quote_spanned!(span=> {
            #veecle_telemetry_crate::future::FutureExt::with_span(
                #async_block,
                #veecle_telemetry_crate::Span::new(#name, #properties),
            ).await
        }))?
    } else {
        syn::parse2(quote_spanned!(span=> {
            let __guard__ = #veecle_telemetry_crate::Span::new(#name, #properties).entered();
            #(#stmts)*
        }))?
    };

    Ok(Block {
        brace_token: block.brace_token,
        stmts: wrapper.stmts,
    })
}

/// Returns a path to the `veecle_telemetry` crate for use when macro users don't set it explicitly.
fn veecle_telemetry_path() -> syn::Result<syn::Path> {
    proc_macro_crate::crate_name("veecle-telemetry")
        .map(|found| match found {
            proc_macro_crate::FoundCrate::Itself => {
                // The only place we use `veecle-telemetry` within "itself" is doc-tests, where it needs to be an external
                // path anyway.
                syn::parse_quote!(::veecle_telemetry)
            }
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                syn::parse_quote!(::#ident)
            }
        })
        .or_else(|_| {
            proc_macro_crate::crate_name("veecle-os").map(|found| match found {
                proc_macro_crate::FoundCrate::Itself => {
                    todo!("unused currently, not sure what behavior will be wanted")
                }
                proc_macro_crate::FoundCrate::Name(name) => {
                    let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                    syn::parse_quote!(::#ident::telemetry)
                }
            })
        })
        .map_err(|_| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "could not find either veecle-telemetry or veecle-os crates",
            )
        })
}
