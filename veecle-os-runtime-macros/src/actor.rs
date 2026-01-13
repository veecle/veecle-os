use std::collections::HashSet;

use heck::ToUpperCamelCase;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Paren;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{Error, FnArg, ItemFn, Lifetime, Meta, Type, TypePath};

/// Parses the arguments inside the `#[actor(...)]` attribute itself.
pub(crate) struct ActorMeta {
    veecle_os_runtime: Option<syn::Path>,
}

impl syn::parse::Parse for ActorMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut veecle_os_runtime = None;

        // The macro input `TokenStream` is only the `...` in `#[actor(...)]`, we expect it to be a standard
        // `syn::MetaList`-like.
        for meta in input.parse_terminated(syn::Meta::parse, syn::Token![,])? {
            if meta.path().is_ident("crate") {
                if let Some((_span, _)) = &veecle_os_runtime {
                    // TODO: attach original span to error diagnostic
                    return Err(Error::new_spanned(
                        meta,
                        "setting `crate` argument multiple times",
                    ));
                }

                let syn::Meta::NameValue(syn::MetaNameValue { value, .. }) = &meta else {
                    return Err(Error::new_spanned(
                        meta,
                        "`crate` must be a name value pair (`crate = veecle_os_runtime`)",
                    ));
                };

                let syn::Expr::Path(syn::ExprPath {
                    attrs: _,
                    qself: None,
                    path,
                }) = value
                else {
                    return Err(Error::new_spanned(
                        value,
                        "invalid value for `crate`, must be a simple path",
                    ));
                };

                veecle_os_runtime = Some((meta.span(), path.clone()));
            } else {
                return Err(Error::new_spanned(meta, "unknown attribute argument"));
            }
        }

        // Default to assuming a non-renamed extern-crate if not set.
        let veecle_os_runtime = veecle_os_runtime.map(|(_, path)| path);

        Ok(Self { veecle_os_runtime })
    }
}

/// Initialize with a set of generic parameters, then visit with type expressions where those generic parameters may be
/// used to detect which are used, afterwards the leftover `Ident`s will be the unused generic parameters.
#[derive(Default)]
struct UnusedGenerics {
    types: HashSet<syn::Ident>,
}

impl Visit<'_> for UnusedGenerics {
    fn visit_type_path(&mut self, ty: &syn::TypePath) {
        if let Some(ident) = ty.path.get_ident() {
            self.types.remove(ident);
        }
        syn::visit::visit_type_path(self, ty);
    }
}

pub fn impl_actor(
    meta: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let ActorMeta { veecle_os_runtime } = syn::parse2(meta)?;
    let veecle_os_runtime = veecle_os_runtime
        .map(Ok)
        .unwrap_or_else(crate::veecle_os_runtime_path)?;
    let mut parsed_item: ItemFn = syn::parse2(item)?;

    let docs = parsed_item.attrs.iter().filter(|attr| match &attr.meta {
        Meta::NameValue(meta_name_value) => meta_name_value.path.is_ident("doc"),
        _ => false,
    });

    let function_name = parsed_item.sig.ident.clone();
    let struct_name = syn::Ident::new(
        &parsed_item.sig.ident.to_string().to_upper_camel_case(),
        function_name.span(),
    );
    let mut request = vec![];
    let mut argument_names = vec![];
    let mut init_context = None;

    let mut unused_generics = UnusedGenerics::default();

    let mut generics = parsed_item.sig.generics.clone();
    generics.where_clause = None;
    let where_clause = parsed_item.sig.generics.where_clause.clone();

    parsed_item.sig.generics = syn::Generics::default();

    let mut generic_args = syn::AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: syn::Token![<](Span::call_site()),
        args: Result::<_, syn::Error>::from_iter(generics.params.iter().map(|param| {
            match param {
                syn::GenericParam::Lifetime(lt) => Err(syn::Error::new_spanned(
                    lt,
                    "#[actor] functions cannot be generic over a lifetime",
                )),
                syn::GenericParam::Type(ty) => {
                    unused_generics.types.insert(ty.ident.clone());
                    Ok(syn::GenericArgument::Type(
                        syn::TypePath {
                            qself: None,
                            path: ty.ident.clone().into(),
                        }
                        .into(),
                    ))
                }
                syn::GenericParam::Const(c) => Ok(syn::GenericArgument::Const(
                    syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: c.ident.clone().into(),
                    }
                    .into(),
                )),
            }
        }))?,
        gt_token: syn::Token![>](Span::call_site()),
    };

    let actor_lifetime = Lifetime::new(
        "'__dont_use_internal_actor_macro_lifetime",
        Span::mixed_site(),
    );

    generics
        .params
        .insert(0, syn::LifetimeParam::new(actor_lifetime.clone()).into());
    generic_args
        .args
        .insert(0, syn::GenericArgument::Lifetime(actor_lifetime.clone()));

    let mut next_argument_name = {
        let mut counter = 0;
        move || {
            counter += 1;
            syn::Ident::new(&format!("arg{counter}"), Span::call_site())
        }
    };

    for argument in parsed_item.sig.inputs.iter_mut() {
        match argument {
            FnArg::Receiver(receiver) => {
                return Err(Error::new(
                    receiver.span(),
                    "method signature may not contain self",
                ));
            }
            FnArg::Typed(typed_argument) => {
                let argument_name = next_argument_name();
                argument_names.push(argument_name.clone());

                unused_generics.visit_type(&typed_argument.ty);

                // Scan for and remove any `init_context` attribute.
                let mut init_context_found = false;
                typed_argument.attrs.retain(|attr| {
                    if attr.path().is_ident("init_context") {
                        init_context_found = true;
                        false
                    } else {
                        true
                    }
                });

                // We ensure only one attribute can exist per function and if so we extract the
                // associated argument into the context.
                if init_context_found {
                    if init_context.is_some() {
                        return Err(Error::new(
                            typed_argument.ty.span(),
                            "only up to one argument with \"[init_context]\" attribute is allowed",
                        ));
                    }

                    init_context = Some((argument_name.clone(), typed_argument.ty.clone()));
                } else {
                    let type_error = Err(Error::new(
                        typed_argument.ty.span(),
                        "only \"Reader\", \"ExclusiveReader\", \"InitializedReader\" and \"Writer\" arguments are allowed",
                    ));

                    let Type::Path(argument_type) = typed_argument.ty.as_ref() else {
                        return type_error;
                    };
                    if argument_type.qself.is_some() {
                        return type_error;
                    }

                    let Some(argument_type_name) = argument_type
                        .path
                        .segments
                        .last()
                        .map(|seg| seg.ident.to_string())
                    else {
                        return type_error;
                    };

                    match argument_type_name.as_str() {
                        "Reader" | "ExclusiveReader" | "InitializedReader" | "Writer" => {
                            request.push((argument_name.clone(), argument_type.clone()));
                        }
                        _ => return type_error,
                    }
                }
            }
        }
    }

    let request_argument_names = make_cons_pattern(extract_argument_names(&request));
    let mut lifetime_replacer = ReplaceAnonymousLifetimeWith::new(actor_lifetime.clone());
    let request_argument_types_struct = make_cons_type(
        request
            .iter()
            .map(|(_, ty)| {
                let mut ty = ty.clone();
                lifetime_replacer.visit_type_path_mut(&mut ty);
                ty
            })
            .collect(),
    );

    let phantom_generic_types: Vec<TypePath> = unused_generics
        .types
        .iter()
        .map(|ty| syn::parse_quote!(core::marker::PhantomData<#ty>))
        .collect();
    let phantom_generic_values: Vec<syn::Expr> = phantom_generic_types
        .iter()
        .map(|_| syn::parse_quote!(core::marker::PhantomData))
        .collect();

    // Even if there was no `#[init_context]` argument, we still declare a unit field for it and destructure it, but
    // it's not in `argument_names` so we won't pass it on to the function.
    let (context_name, context_ty) = init_context
        .map(|(name, ty)| {
            let mut ty = (*ty).clone();
            lifetime_replacer.visit_type_mut(&mut ty);
            (name.clone(), ty)
        })
        .unwrap_or_else(|| (syn::parse_quote!(init_context), syn::parse_quote!(())));

    lifetime_replacer.check_errors()?;

    let return_ty = match &parsed_item.sig.output {
        syn::ReturnType::Default => {
            return Err(Error::new(
                parsed_item.sig.span(),
                "#[actor] functions must return a `Result` or `Never`",
            ));
        }
        syn::ReturnType::Type(_, ty) => ty,
    };

    // Re-spanning this reduces the amount of error spam for non-implementing types.
    let error_ty = quote_spanned!(return_ty.span() => <#return_ty as #veecle_os_runtime::__exports::IsActorResult>::Error);

    let visibility = &parsed_item.vis;

    let expanded = quote! {
        #(#docs)*
        #visibility struct #struct_name #generics #where_clause {
            request: #request_argument_types_struct,
            #context_name: #context_ty,
            __phantom_data_private_to_avoid_name_collisions_veecle: (
                core::marker::PhantomData<&#actor_lifetime ()>,
                (#(#phantom_generic_types,)*),
            ),
        }

        impl #generics #struct_name #generic_args #where_clause {
            #parsed_item
        }

        impl #generics #veecle_os_runtime::Actor<#actor_lifetime> for #struct_name #generic_args #where_clause {
            type StoreRequest = #request_argument_types_struct;
            type InitContext = #context_ty;
            type Error = #error_ty;

            fn new(
                request: Self::StoreRequest,
                #context_name: Self::InitContext
            ) -> Self {
                Self {
                    request,
                    #context_name,
                    __phantom_data_private_to_avoid_name_collisions_veecle:  (
                        core::marker::PhantomData,
                        (#(#phantom_generic_values,)*),
                    ),
                }
            }

            async fn run(self) -> core::result::Result<#veecle_os_runtime::Never, Self::Error> {
                let Self {
                    request: #request_argument_names,
                    #context_name,
                    __phantom_data_private_to_avoid_name_collisions_veecle: _,
                } = self;

                let result = Self::#function_name(
                    #(#argument_names,)*
                ).await;
                #[allow(unreachable_code)]
                <#return_ty as #veecle_os_runtime::__exports::IsActorResult>::into_result(result)
            }
        }
    };

    Ok(expanded)
}

/// Extracts the argument name removing the mutability.
///
/// `mut bar:Foo` -> `bar`
fn extract_argument_names(argument_type_pairs: &[(syn::Ident, TypePath)]) -> Vec<syn::Ident> {
    argument_type_pairs
        .iter()
        .map(|(name, _)| name.clone())
        .collect()
}

/// A [`VisitMut`] implementer that replaces anonymous lifetimes with a specific lifetime, while checking that only
/// anonymous or static lifetimes are used.
struct ReplaceAnonymousLifetimeWith {
    lifetime: syn::Lifetime,
    errors: Vec<Error>,
}

impl ReplaceAnonymousLifetimeWith {
    /// Returns a new instance.
    fn new(lifetime: syn::Lifetime) -> Self {
        Self {
            lifetime,
            errors: Vec::new(),
        }
    }

    /// Returns an error with information about any non-static/anonymous lifetimes that were encountered, if any.
    fn check_errors(&mut self) -> Result<(), Error> {
        let mut errors = self.errors.drain(..);
        let Some(mut combined) = errors.next() else {
            return Ok(());
        };
        for error in errors {
            combined.combine(error)
        }
        Err(combined)
    }
}

impl VisitMut for ReplaceAnonymousLifetimeWith {
    fn visit_lifetime_mut(&mut self, lifetime: &mut Lifetime) {
        match lifetime.ident.to_string().as_str() {
            "static" => { /* Leave it alone. */ }

            "_" => {
                *lifetime = self.lifetime.clone();
            }

            _ => {
                self.errors.push(Error::new(
                    lifetime.span(),
                    "lifetimes on actor arguments must be anonymous or static",
                ));
            }
        }
    }

    fn visit_type_reference_mut(&mut self, reference: &mut syn::TypeReference) {
        syn::visit_mut::visit_type_reference_mut(self, reference);

        // An `&T` with no explicit lifetime is the equivalent of `&'_ T` so we need to replace it too.
        if reference.lifetime.is_none() {
            reference.lifetime = Some(self.lifetime.clone());
        }
    }
}

/// Turns a list of names `[a, b, c]` into a cons-list pattern `(c, (b, (a, ())))`.
fn make_cons_pattern(names: Vec<syn::Ident>) -> syn::PatTuple {
    fn pat_ident(ident: syn::Ident) -> syn::PatIdent {
        syn::PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident,
            subpat: None,
        }
    }

    fn pat_tuple(elems: impl IntoIterator<Item = syn::Pat>) -> syn::PatTuple {
        syn::PatTuple {
            attrs: Vec::new(),
            paren_token: Paren::default(),
            elems: Punctuated::from_iter(elems),
        }
    }

    names.into_iter().fold(pat_tuple([]), |tuple, name| {
        pat_tuple([pat_ident(name).into(), tuple.into()])
    })
}

/// Turns a list of types `[A, B, C]` into a cons-list type `(C, (B, (A, ())))`.
fn make_cons_type(types: Vec<TypePath>) -> syn::TypeTuple {
    fn type_tuple(elems: impl IntoIterator<Item = syn::Type>) -> syn::TypeTuple {
        syn::TypeTuple {
            paren_token: Paren::default(),
            elems: Punctuated::from_iter(elems),
        }
    }

    types.into_iter().fold(type_tuple([]), |tuple, ty| {
        type_tuple([ty.into(), tuple.into()])
    })
}
