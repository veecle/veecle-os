//! This crate provides runtime macros.

#![forbid(unsafe_code)]

mod actor;
mod storable;

#[allow(missing_docs)]
// Documentation is defined on the reexport in `../../src/actor.rs` to allow using intra-doc-links.
#[proc_macro_attribute]
pub fn actor(
    meta: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    actor2(meta.into(), item.into()).into()
}

/// `proc_macro2` implementation of [`actor()`] to allow executing outside the compiler.
///
/// The actual implementation is in the module, this just maps any errors into `compile_error!`s to allow using `?` in
/// the implementation while giving the expected infallible function signature.
fn actor2(
    meta: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    actor::impl_actor(meta, item).unwrap_or_else(|error| error.into_compile_error())
}

#[allow(missing_docs)]
// Documentation is defined on the reexport in `../../src/datastore/slot/mod.rs` to allow using intra-doc-links.
#[proc_macro_derive(Storable, attributes(storable))]
pub fn derive_storable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_storable2(input.into()).into()
}

/// `proc_macro2` implementation of [`derive_storable`] to allow executing outside the compiler.
///
/// The actual implementation is in the module, this just maps any errors into `compile_error!`s to allow using `?` in
/// the implementation while giving the expected infallible function signature.
fn derive_storable2(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    storable::impl_derive_storable(input).unwrap_or_else(|error| error.write_errors())
}

/// Returns a path to the `veecle_os_runtime` crate for use when macro users don't set it explicitly.
fn veecle_os_runtime_path() -> syn::Result<syn::Path> {
    proc_macro_crate::crate_name("veecle-os-runtime")
        .map(|found| match found {
            proc_macro_crate::FoundCrate::Itself => {
                // The only place we use `veecle-os-runtime` within "itself" is doc-tests, where it needs to be an external
                // path anyway.
                syn::parse_quote!(::veecle_os_runtime)
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
                    syn::parse_quote!(::#ident::runtime)
                }
            })
        })
        .map_err(|_| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "could not find either veecle-os-runtime or veecle-os crates",
            )
        })
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    #[test]
    fn test_for_code_coverage() -> Result<(), Box<dyn std::error::Error>> {
        for entry in walkdir::WalkDir::new("tests/ui") {
            let entry = entry?;
            if entry.path().extension().unwrap_or_default() == "rs" {
                runtime_macros::emulate_attributelike_macro_expansion(
                    File::open(entry.path())?,
                    &[
                        ("actor", super::actor2),
                        ("veecle_os_runtime::actor", super::actor2),
                        ("veecle_os_runtime_macros::actor", super::actor2),
                    ],
                )?;
                runtime_macros::emulate_derive_macro_expansion(
                    File::open(entry.path())?,
                    &[
                        ("Storable", super::derive_storable2),
                        ("veecle_os_runtime::Storable", super::derive_storable2),
                        (
                            "veecle_os_runtime_macros::Storable",
                            super::derive_storable2,
                        ),
                    ],
                )?;
            }
        }

        Ok(())
    }
}
