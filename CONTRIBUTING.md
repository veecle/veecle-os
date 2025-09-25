# Contributing

## Contributing Workflow

We welcome contributions from the community!
Follow this workflow to ensure smooth review and integration.

### Before You Start

1. Browse existing work: Check [issues](https://github.com/veecle/veecle-os/issues) to find tasks that need help.
   Look for labels like `good first issue` or `help wanted` for beginner-friendly contributions.
2. Discuss large changes: For significant features or architectural changes, use [GitHub Discussions](https://github.com/veecle/veecle-os/discussions) to discuss the approach with maintainers.
3. Claim your work: Comment on issues to indicate you're working on them to avoid duplicate effort.
4. Read our guidelines: Familiarize yourself with the [Code of Conduct](https://github.com/veecle/veecle-os?tab=coc-ov-file#readme) and this contribution guide.

### Developer certificate of origin (DCO)

To make a good faith effort to ensure licensing criteria are met, this repository requires a DCO process to be followed.
You must sign-off the DCO that you can see at <https://developercertificate.org/> to contribute to the repository by adding a sign-off to your commits.
Use `git commit -s` to sign off your commits.
Refer to [the Developer Certificate of Origin GitHub app](https://probot.github.io/apps/dco/) for details on how this repository enforces sign-off.

### Submitting Changes

1. Make your changes following the code quality guidelines below.
   - `cargo test` at the workspace root will check everything necessary, you may want to check just the subsets you're working on for faster feedback.
2. Commit with [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format.
   We use squash-merge with the PR title and description as commit title and message.
   Please format your PR title in accordance with Conventional Commits.
3. Push your branch to your fork and open a Pull Request.
4. Address feedback from maintainers as needed.

### Getting Help

- **GitHub Discussions**: For questions, ideas, and community support.
- **Issue comments**: For specific questions on relevant issues.
- **Documentation**: Check the [user manual](https://veecle.github.io/veecle-os/user-manual/).
- **Style guide**: Check the [style guide](https://veecle.github.io/style-guide/).

## Supported Platforms

This repository supports the following development environments:

* NixOS (latest stable version) on x86-64
* Fedora (currently supported versions) on x86-64
* Debian stable on x86-64
* macOS (latest stable version) on AArch64

All processes described in the documentation should work on the supported platforms and have sufficient documentation.
If you find an issue on a supported platform, please file a bug or fix via PR.

Some components might support a different set of platforms.
Check the component documentation to learn the supported platforms for a component.

Other platforms are supported on a best effort basis.

We do not audit Windows for `safe-to-deploy`.

## Setup

### Basic dependencies

#### Debian 12 / Ubuntu 20.04

```
apt install -y build-essential        # basic compilers and headers
apt install -y curl                   # for rustup
apt install -y cmake libboost-all-dev # for SOME/IP testing
apt install -y libclang-dev
```

Debian 12 does not have a recent enough version of CMake in the main repository.
However, Debian backports contains a sufficient version.

#### Fedora

```
# Fedora 40 (dnf 4)
dnf group install "C Development Tools and Libraries" "Development Tools"  # basic compilers and headers
# Fedora 41 (dnf 5)
dnf group install c-development development-tools
# Same for both dnf versions:
dnf install clang-devel
dnf install cmake boost-devel # for SOME/IP testing
```

#### Nix/NixOS

There is a `flake.nix` and `shell.nix` with all the required tooling, see comments in those files for more information and some useful commands.

### Arm toolchain

Run the following command from the directory where you want to install the ARM toolchain.

```
curl -L https://developer.arm.com/-/media/Files/downloads/gnu/13.3.rel1/binrel/arm-gnu-toolchain-13.3.rel1-$(arch)-arm-none-eabi.tar.xz | tar xJ
```

```
export BINDGEN_EXTRA_CLANG_ARGS_thumbv7em_none_eabihf="-I$(pwd)/arm-gnu-toolchain-13.3.rel1-x86_64-arm-none-eabi/lib/gcc/arm-none-eabi/13.3.1/include/ -I$(pwd)/arm-gnu-toolchain-13.3.rel1-x86_64-arm-none-eabi/arm-none-eabi/include/"
export PATH="$(pwd)/arm-gnu-toolchain-13.3.rel1-x86_64-arm-none-eabi/bin:$PATH"
```

### Rust

[Install Rust using rustup](https://rustup.rs/).

#### Version

The MSRV (minimum supported Rust version) is documented in the Cargo.toml file in the repository root.
Nightly features are discouraged but if any required addition to the code is impossible otherwise, the move to unstable Rust can be discussed.

### Tools

Tools required to run checks:

* [cargo-deny](https://embarkstudios.github.io/cargo-deny/cli/index.html) for license validation.
  For example, install with `cargo install --locked cargo-deny`.
* [cargo-nextest](https://nexte.st/) as a test runner.
  For example, install with `cargo install --locked cargo-nextest`.
* [`tombi`](https://github.com/tombi-toml/tombi) for TOML formatting and linting.
  For example, install with `cargo install tombi-cli --git https://github.com/tombi-toml/tombi --locked --rev df223791d79a45ee5e32ef6535103f23d5609e11 # --tag v0.6.11`.
  (The version is very important to keep in sync with CI as it affects sort order when formatting).
* [`cargo vet`](https://github.com/mozilla/cargo-vet) to vet dependencies.
  For example, install with `cargo install --locked cargo-vet`.

Optional tools:

* [cargo-about](https://github.com/EmbarkStudios/cargo-about) for generating third-party license notices.
  For example, install with `cargo install --locked cargo-about@=0.7.1`.
* [llvm-cov](https://github.com/taiki-e/cargo-llvm-cov?tab=readme-ov-file#installation) to collect test coverage.
  For example, install with `cargo install cargo-llvm-cov`.
* [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html) to render documentation.
  For example, install with `cargo install mdbook`.
* [`just`](https://just.systems/) for task automation.
  For example, install with `cargo install just`.
* [Vale](https://vale.sh/) for prose linting.
  Refer to the [Vale installation instructions](https://vale.sh/docs/install).
  Refer to our [Vale README](.vale/README.md) for more details.

### Submodules

This repository contains git submodules (see [`submodules`](external/submodules)).

Submodules are not automatically updated during `git pull`, which may cause problems if any of the submodules gets outdated.
You might need to run the following command to update them:

```bash
git submodule update --init --recursive
```

Also consider using [the `submodule.recurse` Git configuration option to automate submodule updates](https://git-scm.com/docs/git-config#Documentation/git-config.txt-submodulerecurse).

## Using Just

This project contains a [`Justfile`](./Justfile) that you can use to run tasks.
Execute `just -l` to list available tasks.

Any non-trivial development procedure should exist as a task in the `Justfile`.
(For example, `cargo clippy` is trivial and does not require a task.
Generating the entire documentation requires multiple commands, so the `Justfile` contains a task for this purpose.)

The published documentation uses `cfg(docsrs)` to enable additional features only available on nightly Rust.
To achieve the same documentation output, set the default Rust toolchain for the repository root to the toolchain defined in the `rust-toolchain-nightly.toml` file.
Additionally, the environment variable `RUSTDOCFLAGS="--cfg docsrs"` must be set.

## EditorConfig

This project contains an [`.editorconfig`](.editorconfig) file to enforce some format considerations.
Because EditorConfig is not enforced in CI, please [configure EditorConfig support in your editor](https://editorconfig.org/#download).

## Code Quality Checks

Refer to [the workflows documentation](.github/README_GITHUB.md) for details.

This repository runs a few code formatters and quality checks in CI on all PRs, it's recommended to run these locally before pushing to avoid failing checks.
These are run within tests as part of the `workspace` crate, so if you are running a general `cargo test --workspace` they will be checked, or you can run `cargo test -p workspace` to run them individually.

Miri is only available on nightly versions of Rust.
To run Miri tests, set the default Rust toolchain for the repository root to the toolchain defined in the `rust-toolchain-nightly.toml` file.
Then run:

```shell
cargo nextest run -p workspace --no-fail-fast --run-ignored all -- miri
```

Code coverage is checked using [`llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov).
This can be run with `just coverage`.
CI uses a nightly version of Rust to generate the coverage data.
To achieve the same coverage output, set the default Rust toolchain for the repository root to the toolchain defined in the `rust-toolchain-nightly.toml` file.

## Adding additional workspaces

The `workspace` crate does workspace detection at compile-time.
To avoid cargo watching the whole repository and rebuilding the `build.rs` file for the `workspace` crate with every change, only `Cargo.toml` at workspace roots are tracked.
Making the `workspace` crate aware of a newly added workspace (or crate outside of any existing workspace), the `workspace` crate needs to be rebuilt.
The easiest way to do so is running `cargo clean -p workspace` at the repository root.

## Blessing Tests

Some tests for generated code have exemplars committed into the repository.
These are expected to change if you touch the related code, if the diff looks as you expect then run `just bless` to save the updated output.
(If you add a new test case then you can also use `just bless` to generate an initial output from it).

## YAML

YAML is formatted via [yamllint][yamllint].
Currently, YAML is only used for GitHub Actions.
To check formatting, run the following command in the root of the repository (with installed yamllint).

```shell
yamllint .
```

[yamllint]:https://github.com/adrienverge/yamllint

## Releasing

The `.github/workflows/release.yaml` GitHub workflow encapsulates most parts of the release process.
The release process uses [`cargo-workspaces`](https://github.com/pksunkara/cargo-workspaces) to automate the release of multiple workspace crates.

### Releasing prerelease versions

Refer to [the Cargo documentation about pre-releases](https://doc.rust-lang.org/cargo/reference/resolver.html#pre-releases).

1. Browse to <https://github.com/veecle/veecle-os/actions/workflows/release.yaml>.
2. Select "run workflow" with the following parameters:
   * "Use workflow from": "Branch: main"
   * "The part of the version number that should be bumped": This won't be taken into account as long as it's a pre-release.
   * "Whether it's a production release or a pre-release": "pre-release"
   * "Require conventional commits.": Unchecked (TODO: this is temporary until we have our first release)

This releases a version `$version-nightly.$timestamp.0`, where `$version` is `current_version + 0.0.1`.
The release does not create any tags or version bumps.

### Releasing production releases

1. Prepare the release notes for the release, review any missing information and perform any necessary editing (as PRs to the `main` branch).
2. Create a `release-$version` branch in GitHub, where `$version` is the version for the release, not the current version.
3. Edit the unreleased changes header to match the release version as a new commit in the release branch.
4. Browse to <https://github.com/veecle/veecle-os/actions/workflows/release.yaml>.
5. Select "run workflow" with the following parameters:
   * "Use workflow from": the branch from a previous step
   * "Whether it's a production release or a pre-release": "production"
   * "The part of the version number that should be bumped": the kind of bump required to release the desired version.
   * "Require conventional commits.": Unchecked (TODO: this is temporary until we have our first release)

The release process creates a version bump commit and a tag in the release branch and pushes the workspace crates to the registry.
The release process then creates a GitHub release and attaches release artifacts.

TODO: consider protecting the new release branch.
(Discuss whether a pattern protection is preferable.)

1. Copy the release notes from the new version into the GitHub release.
2. Create a new pull request to cherry-pick the commits with the release notes update and the version bump to the `main` branch.

### Troubleshooting

Cargo caches packages.
If a Cargo install caches a version of a crate, then Cargo will keep using the first cached crate.

If you "repeat" a Cargo version, then processes can fail because Cargo uses the wrong cached crate.
Clear the cache in `~/.cargo` to address these issues.
