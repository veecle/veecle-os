use anyhow::Result;
use can_dbc::Dbc;
use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn generate(options: &crate::Options, dbc: &Dbc) -> Result<TokenStream> {
    let crate::Options {
        veecle_os_runtime,
        veecle_os_data_support_can,
        ..
    } = options;

    let (message_names, writer_names) =
        Result::<(Vec<_>, Vec<_>)>::from_iter(dbc.messages.iter().map(|message| {
            let name = syn::parse_str::<syn::Ident>(&message.name.to_pascal_case())?;
            let writer_name = format_ident!("{}_writer", message.name.to_snake_case());
            Ok((name, writer_name))
        }))?;

    // because we're potentially generating non-macro code we want to keep the code clean if the
    // argument is the default
    let actor_args = (!veecle_os_runtime.is_ident("veecle_os_runtime"))
        .then_some(quote!((crate = #veecle_os_runtime)));

    // Clippy allows up to 7 args, we have one reader arg + 1 writer per message
    let allow = (dbc.messages.len() > 6).then_some(quote!(#[allow(clippy::too_many_arguments)]));

    Ok(quote! {
        use #veecle_os_data_support_can::Frame;

        /// An actor that will attempt to parse any [`Frame`] messages and publish the parsed messages.
        ///
        /// If used you must also provide some interface-actor that writes the `Frame`s from your transceiver.
        #[#veecle_os_runtime::actor #actor_args]
        #allow
        pub async fn deserialize_frames(
            mut reader: #veecle_os_runtime::InitializedReader<'_, Frame>,
            #(
                mut #writer_names: #veecle_os_runtime::Writer<'_, #message_names>,
            )*
        ) -> core::convert::Infallible {
            loop {
                let frame = reader.wait_for_update().await.read_cloned();
                match frame.id() {
                    #(
                        #message_names::FRAME_ID => {
                            // TODO: something with errors
                            let Ok(msg) = #message_names::try_from(frame) else { continue };
                            #writer_names.write(msg).await;
                        }
                    )*
                    _ => { /* ignore */ }
                }
            }
        }
    })
}
