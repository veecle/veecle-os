use darling::FromDeriveInput;
use proc_macro2::Ident;
use quote::quote;
use syn::{GenericParam, Generics, Lifetime, Path};

/// Parses the struct/enum that is marked with the `Storable` derive macro.
#[derive(FromDeriveInput)]
#[darling(attributes(storable), supports(any))]
pub struct StorableDerive {
    /// The struct/enum ident.
    ident: Ident,
    /// The type's generics.
    generics: Generics,
    /// The `Storable` data type.
    data_type: Option<syn::Type>,
    /// The name of the Veecle OS crate for renaming.
    #[darling(rename = "crate")]
    veecle_os_runtime: Option<Path>,
}

impl StorableDerive {
    /// Generates the derive implementation.
    fn generate_impl(&self) -> syn::Result<proc_macro2::TokenStream> {
        let lifetimes_without_constraints = self.lifetimes_without_constraints();
        let generic_types_without_constraints = self.generic_types_without_constraints();

        let StorableDerive {
            ident,
            generics:
                Generics {
                    lt_token,
                    params: generic_params,
                    gt_token,
                    where_clause,
                },
            data_type,
            veecle_os_runtime,
        } = self;

        let veecle_os_runtime = veecle_os_runtime
            .clone()
            .map(Ok)
            .unwrap_or_else(crate::veecle_os_runtime_path)?;
        let data_type = data_type.clone().unwrap_or_else(|| syn::parse_quote!(Self));

        Ok(quote!(
            #[automatically_derived]
            impl
            #lt_token #generic_params #gt_token
            #veecle_os_runtime::Storable for #ident
            #lt_token #(#lifetimes_without_constraints,)* #(#generic_types_without_constraints),* #gt_token
            #where_clause
            {
                type DataType = #data_type;
            }
        ))
    }

    /// Provides the lifetimes without constraints for use in the struct/enum position. (`impl<...> for Foo<'here>`).
    fn lifetimes_without_constraints(&self) -> Vec<&Lifetime> {
        self.generics
            .params
            .iter()
            .filter_map(|param| {
                if let GenericParam::Lifetime(lifetime) = param {
                    Some(&lifetime.lifetime)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Provides the generic types without constraints for use in the struct/enum position (`impl<...> for Foo<HERE>`).
    fn generic_types_without_constraints(&self) -> Vec<&Ident> {
        self.generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Lifetime(_) => None,
                GenericParam::Type(type_param) => Some(&type_param.ident),
                GenericParam::Const(const_param) => Some(&const_param.ident),
            })
            .collect()
    }
}

/// Implementation of the `Storable` derive macro.
pub fn impl_derive_storable(
    input: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    let parsed_input = StorableDerive::from_derive_input(&syn::parse2(input)?)?;
    Ok(parsed_input.generate_impl()?)
}
