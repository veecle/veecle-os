extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Ident, Type, parse_macro_input};

pub struct Input {
    crate_path: syn::Ident,
    data: Punctuated<Type, Comma>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let crate_path = input.parse()?;
        let _: syn::Token![,] = input.parse()?;
        let data =
            syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated(input)?;
        Ok(Self { crate_path, data })
    }
}

pub fn impl_create_store_proc(input: TokenStream) -> TokenStream {
    let parsed_input: Input = parse_macro_input!(input as Input);
    let mut idents = Vec::new();
    let mut static_types: Vec<syn::Type> = Vec::new();
    for (index, the_type) in parsed_input.data.iter().enumerate() {
        idents.push(Ident::new(
            &format!("VEECLE_INTERNAL_DATA_STORE_{}", index),
            proc_macro2::Span::call_site(),
        ));
        static_types.push(the_type.clone());
    }

    let veecle_os_runtime = parsed_input.crate_path;

    quote! {
        {
        #[repr(transparent)]
        struct SlotWrapper<T>(#veecle_os_runtime::__exports::Slot<T>)
        where
            T: #veecle_os_runtime::Storable + 'static;

        unsafe impl<T> Sync for SlotWrapper<T> where T: #veecle_os_runtime::Storable + 'static {}

        struct StoreInner(#veecle_os_runtime::__exports::Source);

        impl StoreInner {
            fn match_static<T>(&self) -> &'static #veecle_os_runtime::__exports::Slot<T>
            where
                T: #veecle_os_runtime::Storable,
            {
                #(static #idents: SlotWrapper<#static_types> = SlotWrapper(#veecle_os_runtime::__exports::Slot::<#static_types>::new());)*

                match core::any::TypeId::of::<T>() {
                    #(
                        type_id if type_id == core::any::TypeId::of::<#static_types>() => {
                            (&#idents.0 as &dyn core::any::Any)
                                .downcast_ref()
                                .unwrap()
                        }
                    )*
                    _ => {
                        panic!("no slot available for `{}`", core::any::type_name::<T>())
                    }
                }
            }
        }

        impl<'a> #veecle_os_runtime::__exports::Datastore<'a> for StoreInner {
            fn source(&'a self) -> core::pin::Pin<&'a #veecle_os_runtime::__exports::Source> {
                core::pin::pin!(&self.0)
            }

            fn slot<T>(&self) -> core::pin::Pin<&'static #veecle_os_runtime::__exports::Slot<T>>
            where
                T: #veecle_os_runtime::Storable + 'static,
            {
                core::pin::pin!(self.match_static())
            }
        }

        StoreInner(#veecle_os_runtime::__exports::Source::new())
        }
    }
        .into()
}
