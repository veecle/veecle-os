#![allow(missing_docs, reason = "this is a test crate")]

use std::sync::LazyLock;

use anyhow::{Context, Error, Result};
use colored::Colorize;
use datatest_stable::Utf8Path;
use pretty_assertions::assert_eq;

static BLESS: LazyLock<bool> = LazyLock::new(|| std::env::var("BLESS").as_deref() == Ok("1"));

fn maybe_read_to_string(path: &Utf8Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::new(error)),
    }
}

fn generate_test_case(source_path: &Utf8Path, input: String) -> datatest_stable::Result<()> {
    let source = source_path.file_name().context("missing filename")?;

    let options = veecle_os_data_support_can_codegen::Options {
        veecle_os_runtime: syn::parse_str("::my_veecle_os_runtime")?,
        veecle_os_data_support_can: syn::parse_str("::my_veecle_os_data_support_can")?,
        arbitrary: Some(veecle_os_data_support_can_codegen::ArbitraryOptions {
            path: syn::parse_str("::my_arbitrary")?,
            cfg: Some(syn::parse_str(r#"all()"#)?),
        }),
        serde: syn::parse_str("::my_serde")?,
        message_frame_validations: Box::new(|_| None),
    };

    let mut actual =
        veecle_os_data_support_can_codegen::Generator::new(source, options, &input).into_string();

    actual.insert_str(0, "// editorconfig-checker-disable\n");

    let expected_path = source_path.with_extension("rs");
    let expected = maybe_read_to_string(&expected_path)?.unwrap_or_default();

    if *BLESS {
        if expected != actual {
            std::fs::write(expected_path, actual)?;
        }
    } else {
        // We don't _actually_ want to override user environment variables,
        // but we _do_ want to disable the `isatty` detection,
        // but there's no way to do one without the other.
        colored::control::set_override(true);

        assert_eq!(
            expected,
            actual,
            "\n\n{} {} {}",
            "rerun with",
            "BLESS=1".bold(),
            "if the changes are expected (see CONTRIBUTING.md for more details)"
        );
    }

    Ok(())
}

datatest_stable::harness!({test = generate_test_case, root = "tests/cases", pattern = ".*\\.dbc"},);
