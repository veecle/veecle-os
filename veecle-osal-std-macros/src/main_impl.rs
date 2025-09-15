use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use quote::quote;
use syn::ItemFn;

/// `proc_macro2` implementation of [`main`][super::main()] to allow executing outside the compiler.
///
/// The actual implementation is in this module, this just maps any errors into `compile_error!`s to allow using `?` in
/// the implementation while giving the expected infallible function signature.
pub fn main2(
    attributes: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let input = match ::syn::parse2::<ItemFn>(input) {
        Ok(data) => data,
        Err(error) => return error.to_compile_error(),
    };

    impl_main(attributes, input).unwrap_or_else(|error| error.write_errors())
}

/// `main` attribute macro arguments.
#[derive(Debug, FromMeta)]
struct MacroArguments {
    #[darling(default)]
    telemetry: bool,
}

/// Implementation of the `main` attribute macro.
///
/// See [`main`][macro@super::main] for documentation.
fn impl_main(
    attributes: proc_macro2::TokenStream,
    input: ItemFn,
) -> darling::Result<proc_macro2::TokenStream> {
    let attr_args = NestedMeta::parse_meta_list(attributes)?;
    let args = MacroArguments::from_list(&attr_args)?;

    let mut error_accumulator = darling::error::Accumulator::default();

    if input.sig.asyncness.is_none() {
        error_accumulator
            .push(Error::custom("function must be `async`").with_span(&input.sig.fn_token));
    }

    if input.sig.ident != "main" {
        error_accumulator
            .push(Error::custom("function must be named `main`").with_span(&input.sig.ident));
    }

    let veecle_osal_std_path = error_accumulator.handle(crate::crate_path(
        "veecle-osal-std",
        "veecle_osal_std",
        &["osal", "std"],
    ));
    error_accumulator = error_accumulator.checkpoint()?;
    let veecle_osal_std_path = veecle_osal_std_path.unwrap();

    let input_block = input.block;

    let telemetry_setup = if args.telemetry {
        let veecle_telemetry_path = error_accumulator.handle(crate::crate_path(
            "veecle-telemetry",
            "veecle_telemetry",
            &["telemetry"],
        ));
        error_accumulator = error_accumulator.checkpoint()?;
        let veecle_telemetry_path = veecle_telemetry_path.unwrap();

        quote!(
            // Initialize `veecle-telemetry` with a random execution ID and console JSON exporter.
            #veecle_telemetry_path::collector::set_exporter(
                #veecle_telemetry_path::protocol::ExecutionId::random(
                    &mut #veecle_osal_std_path::reexports::rand::rng(),
                ),
                &#veecle_telemetry_path::collector::ConsoleJsonExporter
            )
            .unwrap();
        )
    } else {
        quote!()
    };

    error_accumulator.finish()?;

    Ok(quote!(
        fn main(){
            #veecle_osal_std_path::reexports::tokio::runtime::Runtime::new().unwrap().block_on(
                async {
                    #telemetry_setup

                    #input_block
                }
            );
        }
    ))
}
