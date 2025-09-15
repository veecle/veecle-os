#![doc(hidden)]
//! Private API, do not use.

mod expand;

/// Loads a file from a path encoded as a literal string, relative to the file in which the literal was written, returns
/// the full path to the loaded file and the content.
// TODO: replace with <https://github.com/rust-lang/rfcs/pull/3200>
fn load(path: &syn::LitStr) -> syn::Result<(String, String)> {
    /// Loads a file from `path` relative to `file`, returns the full path to the loaded file and the content.
    // Split into a separate method for simplified error handling, could be a `try` block one day.
    fn inner(file: &std::path::Path, path: &str) -> std::io::Result<(String, String)> {
        let file = <&camino::Utf8Path>::try_from(file)
            .map_err(camino::FromPathError::into_io_error)?
            .canonicalize_utf8()?;
        let path = file.parent().unwrap().join(path).into_string();

        let source = fs_err::read_to_string(&path)?;

        Ok((path, source))
    }

    let file = path.span().unwrap().local_file().unwrap();

    // If there was an error finding/reading the file, return that as an error pointing to the literal string.
    inner(&file, &path.value()).map_err(|error| syn::Error::new_spanned(path, error))
}

#[proc_macro]
pub fn from_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<expand::Input> {
        // We expect the input to look like
        //
        // path::to::veecle_os_data_support_can ; mod foo ; "some.dbc" ; extra items
        let krate = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let module = input.parse()?;
        let path = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        let mut extra = Vec::new();
        while !input.is_empty() {
            extra.push(input.parse()?);
        }

        // Convert the relative string literal path into the actual file path and source.
        let (path, source) = load(&path)?;

        // Inform the compiler that this proc-macro needs to rerun if this file changes.
        //
        // This is a hacky but stable version of `proc_macro::tracked_path::path(&path)`. Technically the compiler
        // doesn't need to rerun the proc-macro if a file loaded by its output tokens changes, but currently proc-macro
        // expansion isn't that incremental so this works.
        extra.push(syn::parse_quote!(
            const _: &'static [u8] = include_bytes!(#path);
        ));

        Ok(expand::Input {
            krate,
            module,
            context: path,
            source,
            extra,
        })
    }

    syn::parse_macro_input!(input with parse).expand().into()
}

#[proc_macro]
pub fn from_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<expand::Input> {
        // We expect the input to look like
        //
        // path::to::veecle_os_data_support_can ; mod foo ; r#"VERSION .... (dbc file content)"# ; extra items
        let krate = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let module = input.parse()?;
        let source: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let extra = {
            let mut extra = Vec::new();
            while !input.is_empty() {
                extra.push(input.parse()?);
            }
            extra
        };

        // We don't have a separate file path to use as the context for generated errors, instead
        // point them at the start of the literal string inside the source file we're running in.
        let span = source.span().unwrap();
        let (line, col) = (span.line(), span.column());

        Ok(expand::Input {
            krate,
            module,
            context: format!("{}:{line}:{col}", span.file()),
            source: source.value(),
            extra,
        })
    }

    syn::parse_macro_input!(input with parse).expand().into()
}
