/**
 * This is the entrypoint for the webview.
 */

import { type StartupOptions, start } from "#wasm";

let startupOptions: StartupOptions = {
  isPreview: false,
};

const startupOptionsElement = document.getElementById("startup-options");
if (startupOptionsElement != null) {
  startupOptions = JSON.parse(startupOptionsElement.innerText);
}

console.log("Starting veecle-telemetry-ui egui app. Startup options", startupOptions);

await start(startupOptions);

acquireVsCodeApi().postMessage("ready");
