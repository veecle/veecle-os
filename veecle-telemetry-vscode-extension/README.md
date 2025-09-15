# `veecle-telemetry-ui` VS Code Extension

## Running

Requires Node.js v22 and pnpm to be installed.

Node.js can be installed via https://github.com/Schniz/fnm.
pnpm can be installed via Corepack

1. `pnpm install` (install dependencies)
2. `pnpm compile` (optional, but might reveal issues, build JavaScript bundles)
3. Open this folder in VS Code and run the "Run Extension" launch configuration.
4. In the new VS Code instance run the "Veecle Telemetry UI: Open `veecle-telemetry-ui`" command.

### Requirements

1. Node.js 22

   Can be installed via [fnm](https://github.com/Schniz/fnm).
   fnm can be configured to automatically switch node versions when you cd into a directory.

2. pnpm

   Can be installed via Corepack, run `corepack enable`.
   You can configure fnm to do this for you.

   This might change in the future as Corepack will no longer be bundled
   with Node.js at some point in the future.

3. wasm-pack

   Can be installed with `cargo install wasm-pack`.
