#[doc(hidden)]
pub const fn strip_closure_suffix(name: &'static str, async_closure: bool) -> &'static str {
    // `::{{closure}}`
    let name = name.split_at(name.len() - 13).0;

    // Async fns have another `::{{closure}}` added to the name.
    if async_closure {
        name.split_at(name.len() - 13).0
    } else {
        name
    }
}
