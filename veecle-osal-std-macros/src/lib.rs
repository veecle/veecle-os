//! `veecle-osal-std` macros.

mod main_impl;

use proc_macro::TokenStream;

/// Marks an async `main` function as the entrypoint to a Veecle OS application.
///
/// Sets up async support through [`tokio`](https://crates.io/crates/tokio) and
/// optionally initializes `veecle_os::telemetry`.
///
/// ```
/// #[veecle_os::osal::std::main]
/// async fn main() {
///     //...
/// }
/// ```
///
/// # Telemetry
///
/// Telemetry setup can be enabled by setting the `telemetry` argument to `true`.
/// By default, telemetry is disabled (`false`).
///
/// ```
/// #[veecle_os::osal::std::main(telemetry = true)]
/// async fn main() {
///     //...
/// }
/// ```
#[proc_macro_attribute]
pub fn main(attributes: TokenStream, input: TokenStream) -> TokenStream {
    main_impl::main2(attributes.into(), input.into()).into()
}

/// Returns a path to the `crate_name` crate.
///
/// Takes the import name and path within the `veecle-os` crate as additional parameters.
fn crate_path(
    crate_name: &str,
    import_name: &str,
    veecle_os_path: &[&str],
) -> darling::Result<syn::Path> {
    proc_macro_crate::crate_name(crate_name)
        .map(|found| match found {
            proc_macro_crate::FoundCrate::Itself => {
                let ident = syn::Ident::new(import_name, proc_macro2::Span::call_site());
                syn::parse_quote!(::#ident)
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
                    let veecle_os_path: Vec<syn::Ident> = veecle_os_path
                        .iter()
                        .map(|fragment| syn::Ident::new(fragment, proc_macro2::Span::call_site()))
                        .collect();
                    syn::parse_quote!(::#ident::#(#veecle_os_path)::*)
                }
            })
        })
        .map_err(|_| {
            darling::Error::custom(format!(
                "could not find `{crate_name}` or `veecle-os` crate"
            ))
        })
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::main_impl;

    #[test]
    fn test_for_code_coverage() -> Result<(), Box<dyn std::error::Error>> {
        for entry in walkdir::WalkDir::new("tests/ui") {
            let entry = entry?;
            if entry.path().extension().unwrap_or_default() == "rs" {
                runtime_macros::emulate_attributelike_macro_expansion(
                    File::open(entry.path())?,
                    &[
                        ("main", main_impl::main2),
                        ("veecle_osal_std_macros::main", main_impl::main2),
                    ],
                )?;
            }
        }

        Ok(())
    }
}
