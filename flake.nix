# See `shell.nix` for more details.
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }: flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [
        rust-overlay.overlays.default
      ];
    };
  in
  {
    devShells = rec {
      default = stable;

      stable = pkgs.callPackage ./shell.nix {
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      };

      # Can be used to run nightly-only commands like building the docs with `docsrs` or running the miri tests:
      #
      # ```
      # nix develop .#nightly --command env RUSTDOCFLAGS=--cfg=docsrs just build-private-docs
      # nix develop .#nightly --command cargo nextest run -p workspace --no-fail-fast --run-ignored all -- miri
      # ```
      nightly = pkgs.callPackage ./shell.nix {
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain-nightly.toml;
      };

      without-rust = pkgs.callPackage ./shell.nix {};
    };
  });
}
