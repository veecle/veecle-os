# This `shell.nix` has been tested with NixOS 25.05.
#
# You can either use it directly with your system `nixpkgs`, or via the flake which has pinned `nixpkgs` and `rust-overlay` versions.
#
# The flake also includes the Rust toolchain in the default shell, while this doesn't (and exports an alternative shell
# without it).
#
# For example, with `nix-direnv` the former would be `use nix` while the latter would be `use flake`
{
  pkgs ? import <nixpkgs> {},
  rust-toolchain ? null,
}:

let
  lib = pkgs.lib;
  stdenv = pkgs.stdenv;

in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    boost186 # Last version that works with `capicxx`.
    cargo-about
    cargo-deny
    cargo-nextest
    cargo-vet
    cmake
    editorconfig-checker
    gcc
    gcc-arm-embedded
    gdb
    just
    mdbook
    ninja
    pkg-config
    rust-toolchain
    tombi
    vale
    wasm-pack
  ] ++ (lib.lists.optional stdenv.hostPlatform.isLinux cargo-llvm-cov); # Only available on Linux systems.

  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

  # Configure embedded utilities for bindgen/cargo/cmake etc.
  CC_thumbv6m_none_eabi = "arm-none-eabi-gcc";
  CC_thumbv7m_none_eabi = "arm-none-eabi-gcc";
  CC_thumbv7em_none_eabihf = "arm-none-eabi-gcc";

  CFLAGS_thumbv7em_none_eabihf = "-mfloat-abi=hard";

  BINDGEN_EXTRA_CLANG_ARGS_thumbv7em_none_eabihf = [
    "-isystem ${pkgs.gcc-arm-embedded}/lib/gcc/arm-none-eabi/${pkgs.lib.getVersion pkgs.gcc-arm-embedded}/include"
    "-isystem ${pkgs.gcc-arm-embedded}/arm-none-eabi/include"
  ];

  # Make sure bindgen has all the flags needed to find the host includes.
  "BINDGEN_EXTRA_CLANG_ARGS_${lib.strings.replaceStrings ["-"] ["_"] stdenv.hostPlatform.config}" = let
    optStr = lib.optionalString;
    isClang = stdenv.cc.isClang;
    isGNU = stdenv.cc.isGNU;
    ccVersion = lib.getVersion stdenv.cc.cc;
  in [
      (builtins.readFile "${stdenv.cc}/nix-support/libc-crt1-cflags")
      (builtins.readFile "${stdenv.cc}/nix-support/libc-cflags")
      # For some reason this has `/include-fixed` for the gcc specific includes, but they're in `/include`.
      (lib.strings.removeSuffix "-fixed" (lib.strings.trim (builtins.readFile "${stdenv.cc}/nix-support/libc-cflags")))
      (builtins.readFile "${stdenv.cc}/nix-support/cc-cflags")
      (builtins.readFile "${stdenv.cc}/nix-support/libcxx-cxxflags")
      (optStr isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${ccVersion}/include")
      (optStr isGNU "-isystem ${stdenv.cc.cc}/include/c++/${ccVersion}")
      (optStr isGNU "-isystem ${stdenv.cc.cc}/include/c++/${ccVersion}/${stdenv.hostPlatform.config}")
      # This has a different version number for some reason, it's handled by the removed suffix on `libc-cflags`.
      # (optStr isGNU "-idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${ccVersion}/include")
  ];

  # Runtime dependencies required for `veecle-telemetry-ui`.
  LD_LIBRARY_PATH = lib.makeLibraryPath (with pkgs; [
    wayland
    libglvnd
    libxkbcommon
    vulkan-loader
  ]);

  # By default this sets some options like _FORTIFY_SOURCE that don't work when building debug binaries.
  # Disable them all since this is just a dev-environment.
  NIX_HARDENING_ENABLE = "";
}
