# List all available commands.
default:
    just --list

# Run all tests.
test:
    cargo nextest r

# Generate third-party license notices
build-third-party-notices:
    @mkdir -p target
    cargo about generate \
        --config docs/generate-third-party-notices/about.toml \
        --output-file ./target/third-party-notices.md \
        docs/generate-third-party-notices/about.hbs.md

# Build the user manual.
build-user-manual: build-public-rustdoc build-third-party-notices
    cargo run -p generate-markdown-index ./rustdoc/ >target/rustdoc_index.md
    mdbook build docs/user-manual
    rsync -a --delete target/doc/ docs/user-manual/book/rustdoc

# Build the public rustdoc
build-public-rustdoc:
    cargo doc --no-deps --all-features \
        --package veecle-os \
        --package veecle-osal-api \
        --package veecle-osal-embassy \
        --package veecle-osal-freertos \
        --package veecle-osal-std \
        --package veecle-os-runtime \
        --package veecle-os-data-support-can \
        --package veecle-os-data-support-someip \
        --package veecle-os-test \
        --package veecle-telemetry

# Build the user manual and expose them at http://0.0.0.0:8000/
serve-user-manual: build-user-manual
    python3 -m http.server -d docs/user-manual/book

# Build the private docs
build-private-docs:
    cargo doc --workspace --all-features --no-deps --document-private-items

# Bump cargo dependencies.
bump-cargo-dependencies:
    if ! which cargo-upgrade; then echo "Install cargo-edit"; exit 1; fi
    ./cargo-in-all-workspaces update
    ./cargo-in-all-workspaces upgrade

# Run "cargo clean" in all workspaces.
clean-all-workspaces:
    ./cargo-in-all-workspaces clean

# Run Vale on all Markdown and Rust source
vale-all:
    .vale/check

# Generates coverage reports in Codecov's and cargo-llvm-cov JSON formats.
# This requires nightly, you can cp rust-toolchain-nightly.toml rust-toolchain.toml
coverage *args='--workspace':
    cargo llvm-cov nextest {{args}} --all-features --no-report -E "not (test(trybuild) | test(veecle-os-examples))"
    cargo llvm-cov --doc {{args}} --all-features --no-report
    cargo llvm-cov report --doctests --codecov --output-path codecov.json
    cargo llvm-cov report --doctests --json --output-path cov.json
    cargo llvm-cov report --doctests --html

# Update lockfiles for all workspaces.
update-lockfiles:
    ./cargo-in-all-workspaces metadata --format-version=1 >/dev/null

validate-lockfiles: update-lockfiles
    git diff --exit-code --color=always -- $(git ls-files -- '**/Cargo.lock' 'Cargo.lock')

# Update generated test outputs
bless:
    BLESS=1 cargo test -p veecle-os-data-support-can-codegen --test generate
    TRYBUILD=overwrite cargo test -p veecle-os-runtime --test trybuild
    TRYBUILD=overwrite cargo test -p veecle-os-runtime-macros --test trybuild
    TRYBUILD=overwrite cargo test -p veecle-osal-std-macros --test trybuild
    TRYBUILD=overwrite cargo test -p veecle-osal-api --test trybuild

build-submodule-doc:
    ./docs/generate-submodule-md >target/submodules.md

build-release-archive: build-user-manual coverage build-submodule-doc
    just veecle-telemetry-ui/build-wasm
    rm -rf target/release-archive
    mkdir -p target/release-archive
    cp -a docs/user-manual/book/ target/release-archive/user-manual
    cp -a target/llvm-cov/html/ target/release-archive/coverage
    cp -a veecle-telemetry-ui/dist/ target/release-archive/veecle-telemetry-ui
    cp target/submodules.md target/release-archive/
    cp CHANGELOG.md target/release-archive/
    rustdoc docs/index.md --output target/release-archive/
    ( cd target/ && zip ../release-archive.zip -r release-archive/ )
