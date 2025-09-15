# docs

This directory contains our books.

Run `just serve-*` to render a book and start a web server that displays the book.
Restart the process to apply changes.

## rustdoc integration

Processes that build books build rustdoc if needed.

This process uses the `generate-markdown-index` crate to create `target/rustdoc_index.md`.
The index contains a list of [link reference definitions](https://spec.commonmark.org/0.31.2/#link-reference-definitions) to all elements in the crate documentation.

To create a link to the crate documentation, include the index in your Markdown document:

```
{{#include ../../../target/rustdoc_index.md}}
```

Then you can write :

```
[`Storable`][`trait@veecle_os_runtime::Storable`]
```

To create a link to the `Storable` trait, with `Storable` as the link text.
