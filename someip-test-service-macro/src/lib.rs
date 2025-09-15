//! Macro that helps setup SOME/IP integration test with less boilerplate.

extern crate proc_macro;

use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Attribute macro that sets up SOME/IP integration tests against an external test service.
///
/// This macro automatically launches an instance of the test service before executing each test
/// and terminates it once the test concludes, regardless of the outcome. The test service instance
/// is injected into your test function as an argument, allowing you to communicate with it.
///
/// Each test service instance runs on a distinct network configuration, enabling parallel test execution.
///
/// ## Configuration
///
/// - `logging_level` - Verbosity level of the internal logging. Allowed values: `fatal`, `error`, `warning`, `info`,
///   `debug`, `trace`. Default: `info`.
/// - `timeout_ms` - The test execution timeout in milliseconds. Default: `240000`.
#[proc_macro_attribute]
pub fn test_with_test_service(meta: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let function_attributes = &function.attrs;
    let function_arguments = &function.sig.inputs;
    let function_name = &function.sig.ident;
    let function_visibility = &function.vis;
    let function_block = &function.block;

    let nested_meta_list = match NestedMeta::parse_meta_list(meta.into()) {
        Ok(nested_meta_list) => nested_meta_list,
        Err(error) => return TokenStream::from(Error::from(error).write_errors()),
    };
    let macro_arguments = match MacroArguments::from_list(&nested_meta_list) {
        Ok(macro_arguments) => macro_arguments,
        Err(error) => return TokenStream::from(error.write_errors()),
    };
    let logging_level = macro_arguments
        .logging_level
        .unwrap_or(String::from("info"));
    let timeout_ms = macro_arguments.timeout_ms.unwrap_or(240000);

    let generated_test = quote! {
        #(#function_attributes)*
        #[test]
        #[cfg(target_os = "linux")]
        #[ntest_timeout::timeout(#timeout_ms)]
        #function_visibility fn #function_name() {
            let mut config = someip_test_service::Config::default();
            match #logging_level {
                "fatal" => { config.logging_level = someip_test_service::LoggingLevel::Fatal; },
                "error" => { config.logging_level = someip_test_service::LoggingLevel::Error; },
                "warning" => { config.logging_level = someip_test_service::LoggingLevel::Warning; },
                "info" => { config.logging_level = someip_test_service::LoggingLevel::Info; },
                "debug" => { config.logging_level = someip_test_service::LoggingLevel::Debug; },
                "trace" => { config.logging_level = someip_test_service::LoggingLevel::Trace; },
                _ => panic!("invalid log level provided!"),
            }
            let test_service = someip_test_service::TestService::new(&config);

            let closure = |#function_arguments| { #function_block };
            let result = std::panic::catch_unwind(move || { closure(&test_service) });

            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };

    generated_test.into()
}

#[derive(Debug, FromMeta)]
struct MacroArguments {
    logging_level: Option<String>,
    timeout_ms: Option<u32>,
}
