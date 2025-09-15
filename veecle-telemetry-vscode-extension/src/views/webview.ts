import vscode from "vscode";

import { Disposable } from "#util/dispose";
import type { Document, StartupOptions, ToWebviewMessage } from "#wasm";

export function getWebviewOptions(extensionUri: vscode.Uri): vscode.WebviewOptions {
  return {
    // Enable JavaScript in the webview
    enableScripts: true,

    // And restrict the webview to only loading content from our extension's `media` directory.
    localResourceRoots: [
      vscode.Uri.joinPath(extensionUri, "assets"),
      vscode.Uri.joinPath(extensionUri, "dist", "webview"),
    ],
  };
}

export function getWebviewPanelOptions(): vscode.WebviewPanelOptions {
  return {
    // Don't unload when hidden.
    retainContextWhenHidden: true,
  };
}

export function getWebviewAndPanelOptions(
  extensionUri: vscode.Uri,
): vscode.WebviewOptions & vscode.WebviewPanelOptions {
  return {
    ...getWebviewOptions(extensionUri),
    ...getWebviewPanelOptions(),
  };
}

export class VeecleTelemetryWebviewPanel extends Disposable {
  public constructor(
    protected readonly webviewPanel: vscode.WebviewPanel,
    protected readonly context: vscode.ExtensionContext,
    private readonly options: StartupOptions,
    private document?: Document,
  ) {
    super();

    this.webviewPanel.webview.options = getWebviewOptions(context.extensionUri);

    // Set the webview's initial html content
    this.webviewPanel.webview.html = this.getHtmlForWebview();

    // Send the document when the webview is ready.
    this.register(
      this.webviewPanel.webview.onDidReceiveMessage((e) => {
        console.log("veecle-os ui > webview message", e);

        if (e === "ready") {
          this.update();
        }
      }),
    );
  }

  public updateDocument(document: Document, reloadPage = false) {
    this.document = document;
    this.update(reloadPage);
  }

  public update(reloadPage = false) {
    if (this.isDisposed) {
      return;
    }

    if (reloadPage) {
      this.webviewPanel.webview.html = this.getHtmlForWebview();
      return;
    }

    if (this.document != null) {
      this.postMessage({ type: "document", document: this.document });
    }
  }

  public postMessage(message: ToWebviewMessage): void {
    if (this.isDisposed) {
      return;
    }

    this.webviewPanel.webview.postMessage(message);
  }

  private getWebviewUri(...pathSegments: string[]) {
    return this.webviewPanel.webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, ...pathSegments));
  }

  private getHtmlForWebview() {
    const webview = this.webviewPanel.webview;

    // Map local paths to URIs for the webview.
    const scriptUri = this.getWebviewUri("dist", "webview", "bundle.js");

    const stylesResetUri = this.getWebviewUri("assets", "reset.css");
    const stylesVsCodeUri = this.getWebviewUri("assets", "vscode.css");
    const stylesVeecleTelemetryUri = this.getWebviewUri("assets", "veecle-telemetry-ui.css");

    // Use a nonce to only allow specific scripts to be run
    const nonce = getNonce();

    return `
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="UTF-8">

          <!--
            Use a content security policy to only allow loading images from https or from our extension directory,
            and only allow scripts that have a specific nonce.
          -->
          <meta
            http-equiv="Content-Security-Policy"
            content="
              default-src 'none';
              style-src ${webview.cspSource} 'unsafe-inline';
              img-src ${webview.cspSource} https:;
              script-src 'nonce-${nonce}' 'unsafe-eval';
              connect-src ${webview.cspSource} ws: wss:;
            "
          >

          <!-- Disable zooming: -->
          <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">

          <link href="${stylesResetUri}" rel="stylesheet">
          <link href="${stylesVsCodeUri}" rel="stylesheet">
          <link href="${stylesVeecleTelemetryUri}" rel="stylesheet">

          <title>veecle-telemetry-ui</title>
        </head>
        <body>
          <!-- The WASM code will resize the canvas dynamically -->
          <!-- the id is hardcoded in lib.rs. so, make sure both match. -->
          <canvas id="app"></canvas>

          <div class="centered" id="loading-text">
            <!-- the loading spinner will be removed in lib.rs -->
            <p>
              Loading…
            </p>
            <div class="lds-dual-ring"></div>
          </div>

          <script type="application/json" id="startup-options">${JSON.stringify(this.options)}</script>

          <script nonce="${nonce}" src="${scriptUri}"></script>
        </body>
      </html>
    `;
  }
}

function getNonce() {
  let text = "";
  const possible = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}
