use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, GenericParam, Generics, Lifetime, Path};

/// Parses the struct/enum that is marked with the `Storable` derive macro.
pub struct StorableDerive {
    /// The struct/enum ident.
    ident: Ident,
    /// The type's generics.
    generics: Generics,
    /// The `Storable` data type.
    data_type: Option<syn::Type>,
    /// The name of the Veecle OS crate for renaming.
    veecle_os_runtime: Option<Path>,
}

impl StorableDerive {
    /// Parses a `DeriveInput` to extract storable attributes.
    fn from_derive_input(input: DeriveInput) -> syn::Result<Self> {
        let ident = input.ident;
        let generics = input.generics;

        let mut data_type = None;
        let mut veecle_os_runtime = None;

        // Iterate through attributes to find #[storable(...)]
        for attr in input.attrs {
            if !attr.path().is_ident("storable") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                match meta
                    .path
                    .get_ident()
                    .map(|ident| ident.to_string())
                    .as_deref()
                {
                    Some("data_type") => {
                        if data_type.is_some() {
                            return Err(meta.error("setting `data_type` argument multiple times"));
                        }

                        let value = meta.value()?;

                        // Check if user provided a string literal (was previously supported).
                        if value.peek(syn::LitStr) {
                            let literal = value.parse::<syn::LitStr>()?;
                            let string = literal.value();
                            return Err(syn::Error::new(
                                literal.span(),
                                format!("string literal syntax is not supported, use `data_type = {string}` instead"),
                            ));
                        }

                        let parsed = value.parse::<syn::Type>().map_err(|error| {
                            syn::Error::new(
                                error.span(),
                                format!("expected a type for `data_type` argument: {}", error),
                            )
                        })?;

                        // Check if there are remaining tokens after the type (but not if it's a
                        // comma, which separates attribute arguments).
                        if !value.is_empty() && !value.peek(syn::Token![,]) {
                            return Err(value.error("expected a type for `data_type` argument: found extra tokens after type"));
                        }

                        data_type = Some(parsed);
                    }
                    Some("crate") => {
                        if veecle_os_runtime.is_some() {
                            return Err(meta.error("setting `crate` argument multiple times"));
                        }

                        let value = meta.value()?;

                        // Check if user provided a string literal (was previously supported).
                        if value.peek(syn::LitStr) {
                            let literal = value.parse::<syn::LitStr>()?;
                            let string = literal.value();
                            return Err(syn::Error::new(
                                literal.span(),
                                format!("string literal syntax is not supported, use `crate = {string}` instead"),
                            ));
                        }

                        let parsed = syn::Path::parse_mod_style(value).map_err(|error| {
                            syn::Error::new(
                                error.span(),
                                format!("expected a path for `crate` argument: {}", error),
                            )
                        })?;

                        // Check if there are remaining tokens after the path (but not if it's a
                        // comma, which separates attribute arguments).
                        if !value.is_empty() && !value.peek(syn::Token![,]) {
                            return Err(value.error("expected a path for `crate` argument: found extra tokens after path"));
                        }

                        veecle_os_runtime = Some(parsed);
                    }
                    _ => return Err(meta.error("unknown attribute argument")),
                }

                Ok(())
            })?;
        }

        Ok(Self {
            ident,
            generics,
            data_type,
            veecle_os_runtime,
        })
    }

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
) -> syn::Result<proc_macro2::TokenStream> {
    let parsed_input = StorableDerive::from_derive_input(syn::parse2(input)?)?;
    parsed_input.generate_impl()
}
