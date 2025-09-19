# See `shell.nix` for more details.
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-unstable,
    rust-overlay,
    flake-utils,
  }: flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs-unstable = import nixpkgs-unstable {
      inherit system;
    };
    pkgs = import nixpkgs {
      inherit system;
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          tombi = pkgs-unstable.tombi.overrideAttrs (oldAttrs: rec {
            version = "0.6.10";
            name = "${oldAttrs.pname}-${version}";
            src = final.fetchFromGitHub {
              owner = "tombi-toml";
              repo = "tombi";
              tag = "v${version}";
              hash = "sha256-d3wB5aLv0xTh2n3ESBN6hKjR2qlbOXJs4/4DYyJGn7c=";
            };
            cargoDeps = final.rustPlatform.fetchCargoVendor {
              inherit src name;
              hash = "sha256-7fjvYvftnM6pHr40/uB0kkxuQ2CMPPd8asRgukHUY9k=";
            };
          });
        })
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

    checks = {
      stable-shell = self.devShells.${system}.stable;
      nightly-shell = self.devShells.${system}.nightly;
      without-rust-shell = self.devShells.${system}.without-rust;
    };
  });
}
