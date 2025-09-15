/*
 * Copied / adapted from `vscode`.
 *
 * See License.vscode.txt in the package directory for license information.
 */

import * as vscode from "vscode";
import * as uri from "vscode-uri";

import { Disposable } from "#util/dispose";
import { isTraceFile } from "#util/file";
import { VeecleTelemetryWebviewPanel, getWebviewAndPanelOptions } from "#views/webview";
import type { Document } from "#wasm";

type PreviewWebviewInput = {
  readonly resource: vscode.Uri;
  readonly resourceColumn?: vscode.ViewColumn;
  readonly getTitle?: (resource: vscode.Uri) => string;
};

class VeecleTelemetryPreviewWebview extends VeecleTelemetryWebviewPanel {
  private readonly debounceDelay = 250;
  private debounceTimer: any;
  private firstUpdate = true;

  private currentVersion?: PreviewDocumentVersion;

  public readonly resource: vscode.Uri;

  constructor(
    webviewPanel: vscode.WebviewPanel,
    context: vscode.ExtensionContext,
    private readonly input: PreviewWebviewInput,
  ) {
    super(webviewPanel, context, { isPreview: true });

    this.resource = input.resource;

    this.register(
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (this.isPreviewOf(event.document.uri)) {
          this.refresh();
        }
      }),
    );

    this.register(
      vscode.workspace.onDidOpenTextDocument((document) => {
        if (this.isPreviewOf(document.uri)) {
          this.refresh();
        }
      }),
    );

    const watcher = this.register(
      vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(this.resource, "*")),
    );
    this.register(
      watcher.onDidChange((uri) => {
        if (this.isPreviewOf(uri)) {
          // Only use the file system event when VS Code does not already know about the file
          if (!vscode.workspace.textDocuments.some((doc) => doc.uri.toString() === uri.toString())) {
            this.refresh();
          }
        }
      }),
    );

    this.refresh();
  }

  override dispose() {
    super.dispose();

    clearTimeout(this.debounceTimer);
  }

  public get state() {
    return {
      resource: this.resource.toString(),
      resourceColumn: this.input.resourceColumn,
    };
  }

  /**
   * The first call immediately refreshes the preview,
   * calls happening shortly thereafter are debounced.
   */
  public refresh(forceUpdate = false) {
    // Schedule update if none is pending
    if (!this.debounceTimer) {
      if (this.firstUpdate) {
        this.updatePreview(forceUpdate);
      } else {
        this.debounceTimer = setTimeout(() => this.updatePreview(forceUpdate), this.debounceDelay);
      }
    }

    this.firstUpdate = false;
  }

  public isPreviewOf(resource: vscode.Uri): boolean {
    return this.resource.fsPath === resource.fsPath;
  }

  private async updatePreview(forceUpdate = false): Promise<void> {
    clearTimeout(this.debounceTimer);
    this.debounceTimer = undefined;

    if (this.isDisposed) {
      return;
    }

    let document: vscode.TextDocument;
    try {
      document = await vscode.workspace.openTextDocument(this.resource);
    } catch {
      if (!this.isDisposed) {
        await this.showFileNotFoundError();
      }
      return;
    }

    if (this.isDisposed) {
      return;
    }

    const pendingVersion = new PreviewDocumentVersion(document);
    if (!forceUpdate && this.currentVersion?.equals(pendingVersion)) {
      return;
    }

    const shouldReloadPage = forceUpdate;
    this.currentVersion = pendingVersion;

    this.updateWebviewContent(
      {
        name: document.fileName,
        bytes: document.getText(),
      },
      shouldReloadPage,
    );
  }

  private async showFileNotFoundError() {
    // TODO: implement
    // this.postMessage();
  }

  private updateWebviewContent(document: Document, reloadPage: boolean): void {
    if (this.isDisposed) {
      return;
    }

    if (this.input.getTitle) {
      this.webviewPanel.title = this.input.getTitle(this.resource);
    }

    super.updateDocument(document, reloadPage);
  }
}

export interface IManagedVeecleTelemetryPreview extends Disposable {
  readonly resource: vscode.Uri;
  readonly resourceColumn: vscode.ViewColumn;

  readonly onDispose: vscode.Event<void>;
  readonly onDidChangeViewState: vscode.Event<vscode.WebviewPanelOnDidChangeViewStateEvent>;

  matchesResource(otherResource: vscode.Uri, otherPosition: vscode.ViewColumn | undefined): boolean;
}

export class PreviewDocumentVersion {
  public readonly resource: vscode.Uri;
  private readonly version: number;

  public constructor(document: vscode.TextDocument) {
    this.resource = document.uri;
    this.version = document.version;
  }

  public equals(other: PreviewDocumentVersion): boolean {
    return this.resource.fsPath === other.resource.fsPath && this.version === other.version;
  }
}

export class VeecleTelemetryEditorPreview extends Disposable implements IManagedVeecleTelemetryPreview {
  public static readonly viewType = "veecleTelemetry.editorPreview";

  public static revive(
    resource: vscode.Uri,
    webview: vscode.WebviewPanel,
    context: vscode.ExtensionContext,
  ): VeecleTelemetryEditorPreview {
    return new VeecleTelemetryEditorPreview(webview, resource, context);
  }

  private readonly preview: VeecleTelemetryPreviewWebview;

  private constructor(
    private readonly webviewPanel: vscode.WebviewPanel,
    resource: vscode.Uri,
    context: vscode.ExtensionContext,
  ) {
    super();

    this.preview = this.register(
      new VeecleTelemetryPreviewWebview(this.webviewPanel, context, {
        resource,
      }),
    );

    this.register(
      this.webviewPanel.onDidDispose(() => {
        this.dispose();
      }),
    );

    this.register(
      this.webviewPanel.onDidChangeViewState((e) => {
        this.onDidChangeViewStateEmitter.fire(e);
      }),
    );
  }

  private readonly onDisposeEmitter = this.register(new vscode.EventEmitter<void>());
  public readonly onDispose = this.onDisposeEmitter.event;

  private readonly onDidChangeViewStateEmitter = this.register(
    new vscode.EventEmitter<vscode.WebviewPanelOnDidChangeViewStateEvent>(),
  );
  public readonly onDidChangeViewState = this.onDidChangeViewStateEmitter.event;

  override dispose() {
    this.onDisposeEmitter.fire();
    super.dispose();
  }

  public matchesResource(_otherResource: vscode.Uri, _otherPosition: vscode.ViewColumn | undefined): boolean {
    return false;
  }

  public get resource() {
    return this.preview.resource;
  }

  public get resourceColumn() {
    return this.webviewPanel.viewColumn || vscode.ViewColumn.One;
  }
}

interface DynamicPreviewInput {
  readonly resource: vscode.Uri;
  readonly resourceColumn: vscode.ViewColumn;
}

export class VeecleTelemetryDynamicPreview extends Disposable implements IManagedVeecleTelemetryPreview {
  public static readonly viewType = "veecleTelemetry.preview";

  public readonly resourceColumn: vscode.ViewColumn;

  private preview: VeecleTelemetryPreviewWebview;

  public static revive(
    input: DynamicPreviewInput,
    webview: vscode.WebviewPanel,
    context: vscode.ExtensionContext,
  ): VeecleTelemetryDynamicPreview {
    // webview.iconPath = contentProvider.iconPath;

    return new VeecleTelemetryDynamicPreview(input, webview, context);
  }

  public static create(
    input: DynamicPreviewInput,
    previewColumn: vscode.ViewColumn,
    context: vscode.ExtensionContext,
  ): VeecleTelemetryDynamicPreview {
    const webview = vscode.window.createWebviewPanel(
      VeecleTelemetryDynamicPreview.viewType,
      VeecleTelemetryDynamicPreview.getPreviewTitle(input.resource),
      previewColumn,
      getWebviewAndPanelOptions(context.extensionUri),
    );

    // webview.iconPath = contentProvider.iconPath;

    return new VeecleTelemetryDynamicPreview(input, webview, context);
  }

  constructor(
    input: DynamicPreviewInput,
    private readonly webviewPanel: vscode.WebviewPanel,
    private readonly context: vscode.ExtensionContext,
  ) {
    super();

    this.resourceColumn = input.resourceColumn;

    this.preview = this.createPreview(input.resource);

    webviewPanel.onDidDispose;

    this.register(
      webviewPanel.onDidDispose(() => {
        this.dispose();
      }),
    );

    this.register(
      this.webviewPanel.onDidChangeViewState((e) => {
        this.onDidChangeViewStateEmitter.fire(e);
      }),
    );

    this.register(
      vscode.window.onDidChangeActiveTextEditor((editor) => {
        // Only allow previewing normal text editors which have a viewColumn: See #101514
        if (typeof editor?.viewColumn === "undefined") {
          return;
        }

        if (isTraceFile(editor.document) && !this.preview.isPreviewOf(editor.document.uri)) {
          this.update(editor.document.uri);
        }
      }),
    );
  }

  private readonly onDisposeEmitter = this.register(new vscode.EventEmitter<void>());
  public readonly onDispose = this.onDisposeEmitter.event;

  private readonly onDidChangeViewStateEmitter = this.register(
    new vscode.EventEmitter<vscode.WebviewPanelOnDidChangeViewStateEvent>(),
  );
  public readonly onDidChangeViewState = this.onDidChangeViewStateEmitter.event;

  override dispose() {
    this.preview.dispose();
    this.webviewPanel.dispose();

    this.onDisposeEmitter.fire();
    super.dispose();
  }

  public get resource() {
    return this.preview.resource;
  }

  public reveal(viewColumn: vscode.ViewColumn) {
    this.webviewPanel.reveal(viewColumn);
  }

  public update(newResource: vscode.Uri) {
    if (this.preview.isPreviewOf(newResource)) {
      return;
    }

    this.preview.dispose();
    this.preview = this.createPreview(newResource);
  }

  private static getPreviewTitle(resource: vscode.Uri): string {
    const resourceLabel = uri.Utils.basename(resource);
    return `Preview ${resourceLabel}`;
  }

  public get position(): vscode.ViewColumn | undefined {
    return this.webviewPanel.viewColumn;
  }

  public matchesResource(otherResource: vscode.Uri, otherPosition: vscode.ViewColumn | undefined): boolean {
    return this.position === otherPosition;
  }

  public matches(otherPreview: VeecleTelemetryDynamicPreview): boolean {
    return this.matchesResource(otherPreview.preview.resource, otherPreview.position);
  }

  private createPreview(resource: vscode.Uri): VeecleTelemetryPreviewWebview {
    return new VeecleTelemetryPreviewWebview(this.webviewPanel, this.context, {
      resource,
      resourceColumn: this.resourceColumn,
      getTitle: (resource) => VeecleTelemetryDynamicPreview.getPreviewTitle(resource),
    });
  }
}
