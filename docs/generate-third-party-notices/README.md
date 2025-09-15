# Third-Party License Notice Generation

This directory contains configuration for `cargo-about` to generate third-party license notices.

## Requirements

You need `cargo-about` installed to generate the notices.
See the main CONTRIBUTING.md for installation instructions.

## Usage

```bash
just build-third-party-notices
```

## Files

- `about.toml`: Configuration for `cargo-about` with license policy matching deny.toml
- `about.hbs.md`: Handlebars template for markdown output format
- Output: `../../target/third-party-notices.md` (included in user manual)
