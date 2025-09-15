import * as path from "node:path";

import { defineConfig } from "@rspack/cli";
import { rspack } from "@rspack/core";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";
import { merge } from "webpack-merge";
import type { RuleSetRule } from "@rspack/core";

const nodeTargets = ["maintained node versions and node >= 20"];
const browserTargets = ["chrome >= 109", "edge >= 132", "firefox >= 128", "safari >= 15"];

function swcLoader(targets: string[]): RuleSetRule {
  return {
    test: /\.ts$/,
    exclude: [/node_modules/],
    loader: "builtin:swc-loader",
    options: {
      jsc: {
        parser: {
          syntax: "typescript",
        },
      },
      env: {
        targets,
      },
    },
    type: "javascript/auto",
  };
}

const baseConfig = defineConfig({
  context: __dirname,
  mode: "none", // this leaves the source code as close as possible to the original (when packaging we set this to 'production')
  output: {
    filename: "[name].js",
    path: path.join(__dirname, "./dist"),
  },
  resolve: {
    extensions: [".ts", ".js"],
  },
  externals: {
    // The vscode-module is created on-the-fly and must be excluded.
    vscode: "commonjs vscode",
    // Add other modules that cannot be webpack'ed.
    // Modules added here also need to be added in the .vscodeignore file.
  },
  devtool: "nosources-source-map", // create a source map that points to the original source file
  infrastructureLogging: {
    level: "log", // enables logging required for problem matchers
  },
});

const webExtensionConfig = defineConfig({
  target: "webworker", // extensions run in a webworker context
  entry: {
    "extension-web": "./src/extension.ts",
    "test/web": "./src/test/web.ts",
  },
  output: {
    library: {
      type: "commonjs2",
    },
  },
  module: {
    rules: [swcLoader(browserTargets)],
  },
  resolve: {
    mainFields: ["browser", "module", "main"],
    fallback: {
      // Webpack 5 no longer polyfills Node.js core modules automatically.
      // See https://webpack.js.org/configuration/resolve/#resolvefallback
      // for the list of Node.js core module polyfills.
      assert: require.resolve("assert"),
    },
  },
  plugins: [
    new rspack.optimize.LimitChunkCountPlugin({
      maxChunks: 1, // disable chunks by default since web extensions must be a single bundle
    }),
    new rspack.DefinePlugin({
      "process.platform": JSON.stringify("web"),
      "process.env": JSON.stringify({}),
      "process.env.BROWSER_ENV": JSON.stringify("true"),
    }),
  ],
  performance: {
    hints: false,
  },
});

const nodeExtensionConfig = defineConfig({
  target: "node",
  entry: {
    "extension-node": "./src/extension.ts",
    "test/extension.test": "./src/test/extension.test.ts",
  },
  output: {
    library: {
      type: "commonjs2",
    },
  },
  module: {
    rules: [swcLoader(nodeTargets)],
  },
  resolve: {
    mainFields: ["module", "main"],
  },
});

const webViewConfig = defineConfig({
  target: "web",
  entry: "./src/webview.ts",
  output: {
    filename: "bundle.js",
    path: path.join(__dirname, "./dist/webview"),
    clean: true,
  },
  module: {
    rules: [swcLoader(browserTargets)],
  },
  resolve: {
    mainFields: ["browser", "module", "main"],
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: __dirname,

      outDir: path.resolve(__dirname, "./wasm"),

      watchDirectories: [path.resolve(__dirname, "../veecle-telemetry-ui/src"), path.resolve(__dirname, "../veecle-telemetry-server-protocol/src")],

      extraArgs: "--no-pack",
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
  },
  performance: {
    hints: false,
  },
});

export default [
  merge(baseConfig, webExtensionConfig),
  merge(baseConfig, nodeExtensionConfig),
  merge(baseConfig, webViewConfig),
];
