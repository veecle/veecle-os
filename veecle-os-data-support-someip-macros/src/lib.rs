//! This crate provides SOME/IP macros.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod parse;
mod serialize;

/// Derives implementation of the `Parse` trait.
///
/// ```rust
/// use veecle_os_data_support_someip::parse::{Parse, ParseExt};
///
/// #[derive(Debug, PartialEq, Parse)]
/// struct SomeIpStruct {
///     foo: u16,
///     bar: u32,
/// }
///
/// let bytes = &[0x0, 0x6, 0x1, 0x2, 0x3, 0x4];
/// let deserialized = SomeIpStruct {
///     foo: 6,
///     bar: 0x1020304,
/// };
///
/// let my_struct = SomeIpStruct::parse(bytes).unwrap();
/// assert_eq!(my_struct, deserialized);
/// ```
///
/// It can also derive implementations for structs with a single lifetime parameter:
///
/// ```rust
/// use veecle_os_data_support_someip::parse::{ByteReader, Parse, ParseError, ParseExt};
///
/// #[derive(Debug)]
/// struct Foo;
///
/// impl<'a> Parse<'a> for &'a Foo {
///     fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
///         Ok(&Foo)
///     }
/// }
///
/// #[derive(Debug, Parse)]
/// struct WithLifetimeDerived<'foo> {
///     inner: &'foo Foo,
/// }
///
/// assert!(WithLifetimeDerived::parse(&[]).is_ok());
/// ```
///
/// Zero sized types and tuple structs can be derived as well.
///
/// ```rust
/// use veecle_os_data_support_someip::parse::Parse;
///
/// #[derive(Parse)]
/// struct Zst;
///
/// #[derive(Parse)]
/// struct TupleStruct(u32, u16);
/// ```
///
/// It cannot be derived for enums, unions, or structs with more than one lifetime.
///
/// ```compile_fail
/// use veecle_os_data_support_someip::parse::{Parse};
///
/// #[derive(Parse)]
/// enum Bad {}
///
/// #[derive(Parse)]
/// union AlsoBad {
///   foo: u8,
///   bar: u8,
/// }
///
/// #[derive(Parse)]
/// struct Lively<'a, 'b> {
///   foo: PhantomData<(&'a (), &'b ())>,
/// }
/// ```
#[proc_macro_derive(Parse)]
pub fn someip_parse(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    parse::impl_derive_parse(derive_input).unwrap_or_else(|error| error.into_compile_error().into())
}

/// Derives implementation of the `Serialize` trait.
///
/// ```rust
/// use veecle_os_data_support_someip::serialize::{Serialize, SerializeExt};
///
/// #[derive(Debug, PartialEq, Serialize)]
/// struct SomeIpStruct {
///     foo: u16,
///     bar: u32,
/// }
///
/// let input = SomeIpStruct {
///     foo: 6,
///     bar: 0x1020304,
/// };
/// let bytes = &[0x0, 0x6, 0x1, 0x2, 0x3, 0x4];
///
/// let mut buffer = [0u8; 16];
/// let output = input.serialize(&mut buffer).unwrap();
///
/// assert_eq!(output, bytes);
/// ```
///
/// Zero sized types and tuple structs can be derived as well.
///
/// ```rust
/// use veecle_os_data_support_someip::serialize::Serialize;
///
/// #[derive(Serialize)]
/// struct Zst;
///
/// #[derive(Serialize)]
/// struct TupleStruct(u32, u16);
/// ```
///
/// It cannot be derived for enums or unions.
///
/// ```compile_fail
/// use veecle_os_data_support_someip::serialize::{Serialize};
///
/// #[derive(Serialize)]
/// enum Bad {}
///
/// #[derive(Serialize)]
/// union AlsoBad {
///   foo: u8,
///   bar: u8,
/// }
/// ```
#[proc_macro_derive(Serialize)]
pub fn someip_serialize(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    serialize::impl_derive_serialize(derive_input)
        .unwrap_or_else(|error| error.into_compile_error().into())
}

/// Returns a path to the `veecle_os_data_support_someip` crate.
fn veecle_os_data_support_someip_path() -> syn::Result<syn::Path> {
    proc_macro_crate::crate_name("veecle-os-data-support-someip")
        .map(|found| match found {
            proc_macro_crate::FoundCrate::Itself => {
                syn::parse_quote!(::veecle_os_data_support_someip)
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
                    syn::parse_quote!(::#ident::data_support::someip)
                }
            })
        })
        .map_err(|_| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "could not find either veecle-os-data-support-someip or veecle-os crates",
            )
        })
}
