use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote_spanned;
use syn::DeriveInput;

/// Implementation of the `Serialize` derive macro.
pub fn impl_derive_serialize(derive_input: DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Struct(data_struct) = derive_input.data else {
        return Err(syn::Error::new_spanned(
            &derive_input,
            "Serialize can only be derived for structs",
        ));
    };

    let veecle_os_data_support_someip = crate::veecle_os_data_support_someip_path()?;

    let struct_name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    // ZST
    if data_struct.fields.is_empty() {
        return Ok(quote_spanned! { Span::mixed_site() =>
            impl #impl_generics #veecle_os_data_support_someip::serialize::Serialize for #struct_name #ty_generics #where_clause {
                fn required_length(&self) -> usize {
                    0
                }

                fn serialize_partial(&self, _: &mut #veecle_os_data_support_someip::serialize::ByteWriter) -> Result<(), #veecle_os_data_support_someip::serialize::SerializeError> {
                    Ok(())
                }
            }
        }
        .into());
    }

    let field_names = data_struct
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field
                .ident
                .as_ref()
                .map(|identifier| syn::Member::Named(identifier.clone()))
                .unwrap_or_else(|| {
                    syn::Member::Unnamed(syn::Index {
                        index: index as u32,
                        span: Span::mixed_site(),
                    })
                })
        })
        .collect::<Vec<_>>();
    let field_types = data_struct
        .fields
        .iter()
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    Ok(quote_spanned! { Span::mixed_site() =>
        impl #impl_generics #veecle_os_data_support_someip::serialize::Serialize for #struct_name #ty_generics #where_clause {
            fn required_length(&self) -> usize {
                [#(
                    <#field_types as #veecle_os_data_support_someip::serialize::Serialize>::required_length(&self.#field_names),
                )*].into_iter().sum()
            }

            fn serialize_partial(&self, writer: &mut #veecle_os_data_support_someip::serialize::ByteWriter) -> Result<(), #veecle_os_data_support_someip::serialize::SerializeError> {
                #(
                    <#field_types as #veecle_os_data_support_someip::serialize::Serialize>::serialize_partial(&self.#field_names, writer)?;
                )*

                Ok(())
            }
        }
    }
    .into())
}
