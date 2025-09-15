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
/// # use veecle_telemetry::value::KeyValue;
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

    let function_name = &input.sig.ident;

    // Check for async_trait-like patterns in the block, and instrument the future instead of the wrapper.
    let function_body = match generate_block(
        function_name,
        &input.block,
        input.sig.asyncness.is_some(),
        &arguments,
    ) {
        Ok(body) => body,
        Err(error) => return error.to_compile_error().into(),
    };

    let ItemFn {
        attrs, vis, sig, ..
    } = input;

    let Signature {
        output: return_type,
        inputs: params,
        unsafety,
        constness,
        abi,
        ident,
        asyncness,
        generics:
            Generics {
                params: gen_params,
                where_clause,
                ..
            },
        ..
    } = sig;

    quote::quote!(
        #(#attrs) *
        #vis #constness #unsafety #asyncness #abi fn #ident<#gen_params>(#params) #return_type
        #where_clause
        {
            #function_body
        }
    )
    .into()
}

fn generate_name(
    function_name: &Ident,
    arguments: &Arguments,
    async_closure: bool,
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

        Ok(quote_spanned!(span=>
            #name
        ))
    } else if arguments.short_name {
        let function_name = function_name.to_string();
        Ok(quote_spanned!(span=>
            #function_name
        ))
    } else {
        Ok(quote_spanned!(span=>
            veecle_telemetry::macro_helpers::strip_closure_suffix(core::any::type_name_of_val(&|| {}), #async_closure)
        ))
    }
}

fn generate_properties(arguments: &Arguments) -> proc_macro2::TokenStream {
    if arguments.properties.is_empty() {
        return quote::quote!(&[]);
    }

    let span = arguments.span;
    let properties = arguments
        .properties
        .iter()
        .map(|Property { key, value, span }| {
            quote_spanned!(*span=>
                veecle_telemetry::value::KeyValue::new(#key, #value)
            )
        });
    let properties = Punctuated::<_, Token![,]>::from_iter(properties);
    quote_spanned!(span=>
        &[ #properties ]
    )
}

/// Instrument a block
fn generate_block(
    func_name: &Ident,
    block: &Block,
    async_context: bool,
    arguments: &Arguments,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = generate_name(func_name, arguments, async_context)?;
    let properties = generate_properties(arguments);

    // Generate the instrumented function body.
    // If the function is an `async fn`, this will wrap it in an async block.
    // Otherwise, this will enter the span and then perform the rest of the body.
    if async_context {
        Ok(quote!(
            veecle_telemetry::future::FutureExt::with_span(
                async move { #block },
                veecle_telemetry::Span::new(#name, #properties),
            ).await
        ))
    } else {
        Ok(quote!(
            let __guard__= veecle_telemetry::Span::new(#name, #properties).entered();
            #block
        ))
    }
}
