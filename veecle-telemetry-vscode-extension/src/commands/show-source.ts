import * as vscode from "vscode";

import type { Command } from "#commands/command-manager";
import type { VeecleTelemetryPreviewManager } from "#views/preview-manager";

export class ShowSourceCommand implements Command {
  readonly id: string = "veecleTelemetry.showSource";

  public constructor(private readonly previewManager: VeecleTelemetryPreviewManager) {}

  execute() {
    const { activePreviewResource, activePreviewResourceColumn } = this.previewManager;

    if (activePreviewResource && activePreviewResourceColumn) {
      vscode.workspace.openTextDocument(activePreviewResource).then((document) => {
        return vscode.window.showTextDocument(document, activePreviewResourceColumn);
      });
    }
  }
}
