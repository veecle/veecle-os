//! Generates Veecle OS code from a CAN-DBC file.
//!
//! ```
//! use veecle_os_data_support_can_codegen::{ArbitraryOptions, Generator, Options};
//!
//! let input = include_str!("../tests/cases/CSS-Electronics-SAE-J1939-DEMO.dbc");
//!
//! let options = Options {
//!     veecle_os_runtime: syn::parse_str("veecle_os_runtime")?,
//!     veecle_os_data_support_can: syn::parse_str("veecle_os_data_support_can")?,
//!     arbitrary: Some(ArbitraryOptions {
//!         path: syn::parse_str("arbitrary")?,
//!         cfg: Some(syn::parse_str(r#"feature = "std""#)?),
//!     }),
//!     serde: syn::parse_str("my_serde")?,
//!     message_frame_validations: Box::new(|_| None),
//! };
//!
//! let code = Generator::new("demo.dbc", options, &input).into_string();
//!
//! assert!(code.contains("mod eec1"));
//!
//! # anyhow::Ok(())
//! ```

#![forbid(unsafe_code)]

use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use can_dbc::DBC;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

mod dbc_ext;
mod generate;

/// Options to customize the generated code.
#[derive(Debug)]
pub struct ArbitraryOptions {
    /// A path to the `arbitrary` crate, e.g. `::arbitrary` if it is a dependency of the crate the generated code will
    /// be included in.
    pub path: syn::Path,

    /// Whether and how to cfg-gate the arbitrary usage, if `Some` the code will be gated with `#[cfg]`/`#[cfg_attr]`
    /// using the specified clause.
    pub cfg: Option<syn::Meta>,
}

impl ArbitraryOptions {
    fn to_cfg(&self) -> Option<syn::Attribute> {
        self.cfg
            .as_ref()
            .map(|meta| syn::parse_quote!(#[cfg(#meta)]))
    }
}

/// Options to customize the generated code.
pub struct Options {
    /// A path to the `veecle-os-runtime` crate, e.g. `::veecle_os_runtime` if it is a dependency of the crate the generated code
    /// will be included in.
    pub veecle_os_runtime: syn::Path,

    /// A path to the `veecle-os-data-support-can` crate, e.g. `::veecle_os_data_support_can` if it is a dependency of the crate
    /// the generated code will be included in.
    pub veecle_os_data_support_can: syn::Path,

    /// Whether and how to generate code integrating with `arbitrary`
    pub arbitrary: Option<ArbitraryOptions>,

    /// A path to the `serde` crate, e.g. `::serde` if it is a dependency of the crate the generated code will be
    /// included in.
    pub serde: syn::Path,

    /// For each message name there can be an associated `fn(&Frame) -> Result<()>` expression that
    /// will be called to validate the frame during deserialization.
    #[allow(clippy::type_complexity)]
    pub message_frame_validations: Box<dyn Fn(&syn::Ident) -> Option<syn::Expr>>,
}

impl core::fmt::Debug for Options {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Options")
            .field("veecle_os_runtime", &self.veecle_os_runtime)
            .field(
                "veecle_os_data_support_can",
                &self.veecle_os_data_support_can,
            )
            .field("arbitrary", &self.arbitrary)
            .field(
                "message_frame_validation",
                &format!(
                    "<value of type {}>",
                    core::any::type_name_of_val(&*self.message_frame_validations)
                ),
            )
            .finish()
    }
}

/// Generates Veecle OS code from a CAN-DBC file.
#[derive(Debug)]
pub struct Generator {
    options: Options,
    inner: Result<DBC>,
}

impl Generator {
    /// Constructs a new `Generator` for the given CAN-DBC `input`.
    ///
    /// `context` should be some kind of identifier for error messages, e.g. the source filename.
    pub fn new(context: &str, options: Options, input: &str) -> Self {
        fn parse_dbc(input: &str) -> Result<DBC> {
            DBC::try_from(input).map_err(|error| match error {
                can_dbc::Error::Incomplete(_, rest) => {
                    let parsed = &input[..(input.len() - rest.len())];
                    let lines = parsed.lines().count();
                    anyhow!("parser error around line {lines}")
                }
                can_dbc::Error::Nom(error) => anyhow!(error.to_owned()),
                can_dbc::Error::MultipleMultiplexors => {
                    anyhow!(
                        "can’t Lookup multiplexors because the message uses extended multiplexing"
                    )
                }
            })
        }

        Self {
            options,
            // We don't return the error here so that we can decide later whether to report it via a `Result` or by
            // generating `compile_error!`.
            inner: parse_dbc(input).with_context(|| format!("failed to parse `{context}`")),
        }
    }

    /// Converts the input into a [`TokenStream`], returning any parsing or semantic errors.
    pub fn try_into_token_stream(self) -> Result<TokenStream> {
        generate::generate(&self.options, &self.inner?)
    }

    /// Converts the input into a [`TokenStream`], converting any error into a generated [`compile_error!`].
    pub fn into_token_stream(self) -> TokenStream {
        fn to_compile_error(error: &anyhow::Error) -> TokenStream {
            use std::fmt::Write;

            let mut msg = error.to_string();

            if error.source().is_some() {
                write!(msg, "\n\nCaused by:").unwrap();
                for cause in error.chain().skip(1) {
                    write!(msg, "\n    {cause}").unwrap();
                }
            }

            // Try and use a raw string literal for more readability in the source code, but because there's no good
            // way to make one this could fail depending on the `error` content, so fallback to a non-raw string
            // literal in that case.
            let msg = Literal::from_str(&format!("r#\"\n{msg}\n\"#"))
                .unwrap_or_else(|_| Literal::string(&msg));

            quote!(compile_error!(#msg);)
        }

        match self.try_into_token_stream() {
            Ok(tokens) => tokens,
            Err(error) => to_compile_error(&error),
        }
    }

    /// Converts the input into a formatted code [`String`], returning any parsing or semantic errors.
    pub fn try_into_string(self) -> Result<String> {
        Ok(prettyplease::unparse(
            &syn::parse_file(&self.try_into_token_stream()?.to_string())
                .context("parsing generated code to prettify")?,
        ))
    }

    /// Converts the input into a formatted code [`String`], converting any error into a generated [`compile_error!`].
    pub fn into_string(self) -> String {
        fn maybe_pretty(code: String) -> String {
            match syn::parse_file(&code) {
                Ok(file) => prettyplease::unparse(&file),
                Err(_) => code,
            }
        }

        maybe_pretty(self.into_token_stream().to_string())
    }
}
