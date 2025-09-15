import * as vscode from "vscode";

import { Disposable } from "#util/dispose";
import { VeecleTelemetryWebviewPanel, getWebviewAndPanelOptions, getWebviewOptions } from "#views/webview";

class VeecleTelemetryAppWebview extends VeecleTelemetryWebviewPanel {
  public static readonly viewType = "veecleTelemetry.app";

  public static create(context: vscode.ExtensionContext, column?: vscode.ViewColumn) {
    const panel = vscode.window.createWebviewPanel(
      VeecleTelemetryAppWebview.viewType,
      "veecle-telemetry-ui",
      column ?? vscode.ViewColumn.Active,
      getWebviewAndPanelOptions(context.extensionUri),
    );

    return new VeecleTelemetryAppWebview(panel, context);
  }

  public static revive(webviewPanel: vscode.WebviewPanel, context: vscode.ExtensionContext) {
    // Reset the webview options so we use the latest uri for `localResourceRoots`.
    webviewPanel.webview.options = getWebviewOptions(context.extensionUri);

    return new VeecleTelemetryAppWebview(webviewPanel, context);
  }

  constructor(webviewPanel: vscode.WebviewPanel, context: vscode.ExtensionContext) {
    super(webviewPanel, context, { isPreview: false });

    this.register(
      webviewPanel.onDidDispose(() => {
        this.dispose();
      }),
    );
  }

  public reveal(viewColumn?: vscode.ViewColumn) {
    this.webviewPanel.reveal(viewColumn);
  }

  private readonly onDisposeEmitter = this.register(new vscode.EventEmitter<void>());
  public readonly onDispose = this.onDisposeEmitter.event;

  public dispose() {
    this.webviewPanel.dispose();
    this.onDisposeEmitter.fire();
    super.dispose();
  }
}

export class VeecleTelemetryAppManager extends Disposable implements vscode.WebviewPanelSerializer {
  private activeWebview: VeecleTelemetryAppWebview | undefined = undefined;

  constructor(private readonly context: vscode.ExtensionContext) {
    super();

    this.register(vscode.window.registerWebviewPanelSerializer(VeecleTelemetryAppWebview.viewType, this));
  }

  public openApp() {
    const column = vscode.window.activeTextEditor?.viewColumn ?? undefined;

    if (this.activeWebview) {
      this.activeWebview.reveal(column);
    } else {
      this.activeWebview = VeecleTelemetryAppWebview.create(this.context, column);

      this.activeWebview.onDispose(() => {
        this.activeWebview = undefined;
      });
    }
  }

  public async deserializeWebviewPanel(webviewPanel: vscode.WebviewPanel, _state: unknown) {
    VeecleTelemetryAppWebview.revive(webviewPanel, this.context);
  }

  dispose(): void {
    this.activeWebview?.dispose();

    super.dispose();
  }
}
