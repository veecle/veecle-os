use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote_spanned;
use syn::{DeriveInput, GenericParam, Lifetime, LifetimeParam};

/// Implementation of the `Parse` derive macro.
pub fn impl_derive_parse(derive_input: DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Struct(data_struct) = derive_input.data else {
        return Err(syn::Error::new_spanned(
            &derive_input,
            "Parse can only be derived for structs",
        ));
    };

    let veecle_os_data_support_someip = crate::veecle_os_data_support_someip_path()?;

    let struct_name = &derive_input.ident;

    let padded_generics = match derive_input.generics.lifetimes().count() {
        0 => {
            let mut generics = derive_input.generics.clone();

            generics
                .params
                .push(GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(
                    "'a",
                    Span::call_site(),
                ))));

            generics
        }
        1 => derive_input.generics.clone(),
        _ => {
            return Err(syn::Error::new_spanned(
                &derive_input.generics,
                "Parse can only be derived for structs with no lifetime or a single lifetime",
            ));
        }
    };

    let trait_lifetime = &padded_generics.lifetimes().next().unwrap().lifetime;
    let (_, ty_generics, where_clause) = derive_input.generics.split_for_impl();
    let (impl_generics, _, _) = padded_generics.split_for_impl();

    let field_types = data_struct
        .fields
        .iter()
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    match &data_struct.fields {
        syn::Fields::Named(..) => {
            let field_names = data_struct.fields.iter().map(|field| &field.ident).collect::<Vec<_>>();

            Ok(quote_spanned! { Span::mixed_site() =>
                impl #impl_generics #veecle_os_data_support_someip::parse::Parse< #trait_lifetime > for #struct_name #ty_generics #where_clause {
                    fn parse_partial(reader: &mut #veecle_os_data_support_someip::parse::ByteReader< #trait_lifetime >) -> Result<Self, #veecle_os_data_support_someip::parse::ParseError> {
                        #(
                            let #field_names = <#field_types as #veecle_os_data_support_someip::parse::Parse>::parse_partial(reader)?;
                        )*

                        Ok(Self { #(#field_names),* })
                    }
                }
            }
            .into())
        },
        syn::Fields::Unnamed(..) => {
            Ok(quote_spanned! { Span::mixed_site() =>
                impl #impl_generics #veecle_os_data_support_someip::parse::Parse< #trait_lifetime > for #struct_name #ty_generics #where_clause {
                    fn parse_partial(reader: &mut #veecle_os_data_support_someip::parse::ByteReader< #trait_lifetime >) -> Result<Self, #veecle_os_data_support_someip::parse::ParseError> {
                        Ok(Self (#(
                            <#field_types as #veecle_os_data_support_someip::parse::Parse>::parse_partial(reader)?,
                        )*))
                    }
                }
            }
            .into())
        },
        syn::Fields::Unit => Ok(quote_spanned! { Span::mixed_site() =>
            impl #impl_generics #veecle_os_data_support_someip::parse::Parse< #trait_lifetime > for #struct_name #ty_generics #where_clause {
                fn parse_partial(reader: &mut #veecle_os_data_support_someip::parse::ByteReader< #trait_lifetime >) -> Result<Self, #veecle_os_data_support_someip::parse::ParseError> {
                    Ok(Self)
                }
            }
        }
        .into()),
    }
}
