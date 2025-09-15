use std::collections::HashMap;

use quote::quote;
use veecle_os_data_support_can_codegen::Generator;

pub struct Input {
    pub krate: syn::Path,
    pub module: syn::ItemMod,
    pub context: String,
    pub source: String,
    pub extra: Vec<syn::Item>,
}

#[derive(Default)]
struct Validation {
    message_frames: HashMap<syn::Ident, syn::Expr>,
}

fn extract_validation_functions(items: &mut [syn::Item]) -> Validation {
    use syn::visit_mut::VisitMut;

    struct ValidationVisitor<'a> {
        validation: &'a mut Validation,
        current_message: Option<syn::Ident>,
    }

    impl VisitMut for ValidationVisitor<'_> {
        fn visit_item_impl_mut(&mut self, item_impl: &mut syn::ItemImpl) {
            let ident = if let syn::Type::Path(ty) = &*item_impl.self_ty {
                ty.path.get_ident().cloned()
            } else {
                None
            };

            let previous_message = std::mem::replace(&mut self.current_message, ident);

            syn::visit_mut::visit_item_impl_mut(self, item_impl);

            self.current_message = previous_message;
        }

        fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
            impl_item_fn.attrs.retain(|attr| {
                if attr.path().is_ident("validate_frame") {
                    let Self {
                        validation,
                        current_message,
                    } = self;
                    let fun_ident = &impl_item_fn.sig.ident;
                    validation.message_frames.insert(
                        current_message.clone().unwrap(),
                        syn::parse_quote!(#current_message::#fun_ident),
                    );
                    false
                } else {
                    true
                }
            })
        }
    }

    let mut validation = Validation::default();
    let mut visitor = ValidationVisitor {
        validation: &mut validation,
        current_message: None,
    };

    for item in items {
        visitor.visit_item_mut(item);
    }

    validation
}

impl Input {
    pub fn expand(self) -> proc_macro2::TokenStream {
        let Input {
            krate,
            module,
            context,
            source,
            mut extra,
        } = self;

        let validation = extract_validation_functions(&mut extra);

        let options = veecle_os_data_support_can_codegen::Options {
            veecle_os_runtime: syn::parse_quote!(#krate::reëxports::veecle_os_runtime),
            arbitrary: cfg!(feature = "arbitrary").then(|| {
                veecle_os_data_support_can_codegen::ArbitraryOptions {
                    path: syn::parse_quote!(#krate::reëxports::arbitrary),
                    cfg: None,
                }
            }),
            serde: syn::parse_quote!(#krate::reëxports::serde),
            veecle_os_data_support_can: krate,
            message_frame_validations: Box::new(move |name| {
                validation.message_frames.get(name).cloned()
            }),
        };

        let generated = Generator::new(&context, options, &source).into_token_stream();

        let syn::ItemMod {
            attrs,
            vis,
            unsafety,
            mod_token,
            ident,
            content,
            semi: _,
        } = module;

        assert!(content.is_none());

        quote! {
            #(#attrs)* #vis #unsafety #mod_token #ident {
                #generated
                #(#extra)*
            }
        }
    }
}
