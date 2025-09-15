/*
 * Copied / adapted from `vscode`.
 *
 * See License.vscode.txt in the package directory for license information.
 */

import * as vscode from "vscode";

import { Disposable, disposeAll } from "#util/dispose";
import { type IManagedVeecleTelemetryPreview, VeecleTelemetryDynamicPreview, VeecleTelemetryEditorPreview } from "#views/preview";
import { getWebviewPanelOptions } from "#views/webview";

export interface DynamicPreviewSettings {
  readonly resourceColumn: vscode.ViewColumn;
  readonly previewColumn: vscode.ViewColumn;
}

class WebviewStore<T extends IManagedVeecleTelemetryPreview> extends Disposable {
  private readonly _previews = new Set<T>();

  public override dispose(): void {
    super.dispose();
    for (const preview of this._previews) {
      preview.dispose();
    }
    this._previews.clear();
  }

  [Symbol.iterator](): Iterator<T> {
    return this._previews[Symbol.iterator]();
  }

  public get(resource: vscode.Uri, previewSettings: DynamicPreviewSettings): T | undefined {
    const previewColumn = this._resolvePreviewColumn(previewSettings);
    for (const preview of this._previews) {
      if (preview.matchesResource(resource, previewColumn)) {
        return preview;
      }
    }
    return undefined;
  }

  public add(preview: T) {
    this._previews.add(preview);
  }

  public delete(preview: T) {
    this._previews.delete(preview);
  }

  private _resolvePreviewColumn(previewSettings: DynamicPreviewSettings): vscode.ViewColumn | undefined {
    if (previewSettings.previewColumn === vscode.ViewColumn.Active) {
      return vscode.window.tabGroups.activeTabGroup.viewColumn;
    }

    if (previewSettings.previewColumn === vscode.ViewColumn.Beside) {
      return vscode.window.tabGroups.activeTabGroup.viewColumn + 1;
    }

    return previewSettings.previewColumn;
  }
}

export class VeecleTelemetryPreviewManager
  extends Disposable
  implements vscode.WebviewPanelSerializer, vscode.CustomTextEditorProvider
{
  private readonly dynamicPreviews = this.register(new WebviewStore<VeecleTelemetryDynamicPreview>());
  private readonly editorPreviews = this.register(new WebviewStore<VeecleTelemetryEditorPreview>());

  private _activePreview: IManagedVeecleTelemetryPreview | undefined = undefined;

  constructor(private readonly context: vscode.ExtensionContext) {
    super();

    this.register(
      vscode.window.registerCustomEditorProvider(VeecleTelemetryEditorPreview.viewType, this, {
        webviewOptions: getWebviewPanelOptions(),
      }),
    );

    this.register(vscode.window.registerWebviewPanelSerializer(VeecleTelemetryDynamicPreview.viewType, this));
  }

  public get activePreviewResource() {
    return this._activePreview?.resource;
  }

  public get activePreviewResourceColumn() {
    return this._activePreview?.resourceColumn;
  }

  public async deserializeWebviewPanel(webviewPanel: vscode.WebviewPanel, state: any) {
    try {
      const resource = vscode.Uri.parse(state.resource);
      const resourceColumn = state.resourceColumn;

      const preview = VeecleTelemetryDynamicPreview.revive({ resource, resourceColumn }, webviewPanel, this.context);

      this.registerPreview(preview);
    } catch (e) {
      console.error(e);
    }
  }

  public async resolveCustomTextEditor(
    document: vscode.TextDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken,
  ): Promise<void> {
    const preview = VeecleTelemetryEditorPreview.revive(document.uri, webviewPanel, this.context);

    this.registerEditorPreview(preview);
    this._activePreview = preview;
  }

  public openPreview(resource: vscode.Uri, settings: DynamicPreviewSettings) {
    let preview = this.dynamicPreviews.get(resource, settings);
    if (preview) {
      preview.reveal(settings.previewColumn);
    } else {
      preview = this.createNewPreview(resource, settings);
    }

    preview.update(resource);
  }

  private createNewPreview(resource: vscode.Uri, previewSettings: DynamicPreviewSettings): VeecleTelemetryDynamicPreview {
    const preview = VeecleTelemetryDynamicPreview.create(
      {
        resource,
        resourceColumn: previewSettings.resourceColumn,
      },
      previewSettings.previewColumn,
      this.context,
    );

    this._activePreview = preview;
    return this.registerPreview(preview);
  }

  private registerPreview(preview: VeecleTelemetryDynamicPreview): VeecleTelemetryDynamicPreview {
    this.dynamicPreviews.add(preview);

    preview.onDispose(() => {
      this.dynamicPreviews.delete(preview);
    });

    this.trackActive(preview);

    preview.onDidChangeViewState(() => {
      // Remove other dynamic previews in our column
      disposeAll(
        Array.from(this.dynamicPreviews).filter(
          (otherPreview) => preview !== otherPreview && preview.matches(otherPreview),
        ),
      );
    });
    return preview;
  }

  private registerEditorPreview(preview: VeecleTelemetryEditorPreview): VeecleTelemetryEditorPreview {
    this.editorPreviews.add(preview);

    preview.onDispose(() => {
      this.editorPreviews.delete(preview);
    });

    this.trackActive(preview);
    return preview;
  }

  private trackActive(preview: IManagedVeecleTelemetryPreview): void {
    preview.onDidChangeViewState(({ webviewPanel }) => {
      this._activePreview = webviewPanel.active ? preview : undefined;
    });

    preview.onDispose(() => {
      if (this._activePreview === preview) {
        this._activePreview = undefined;
      }
    });
  }
}
