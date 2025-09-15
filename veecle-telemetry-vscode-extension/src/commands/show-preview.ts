/*
 * Copied / adapted from `vscode`.
 *
 * Licensed under the MIT License.
 * See License.vscode.txt in the package directory for license information.
 */

import * as vscode from "vscode";

import type { Command } from "#commands/command-manager";
import type { VeecleTelemetryPreviewManager } from "#views/preview-manager";

interface ShowPreviewSettings {
  readonly sideBySide?: boolean;
}

async function showPreview(
  previewManager: VeecleTelemetryPreviewManager,
  uri: vscode.Uri | undefined,
  previewSettings: ShowPreviewSettings,
): Promise<any> {
  let resource = uri;
  if (!(resource instanceof vscode.Uri)) {
    if (vscode.window.activeTextEditor) {
      // we are relaxed and don't check for trace files
      resource = vscode.window.activeTextEditor.document.uri;
    }
  }

  if (!(resource instanceof vscode.Uri)) {
    if (!vscode.window.activeTextEditor) {
      // this is most likely toggling the preview
      return vscode.commands.executeCommand("veecleTelemetry.showSource");
    }
    // nothing found that could be shown or toggled
    return;
  }

  const resourceColumn = vscode.window.activeTextEditor?.viewColumn || vscode.ViewColumn.One;

  previewManager.openPreview(resource, {
    resourceColumn: resourceColumn,
    previewColumn: previewSettings.sideBySide ? vscode.ViewColumn.Beside : resourceColumn,
  });
}

export class ShowPreviewCommand implements Command {
  readonly id: string = "veecleTelemetry.showPreview";

  constructor(private readonly previewManager: VeecleTelemetryPreviewManager) {}

  execute(mainUri?: vscode.Uri, allUris?: vscode.Uri[]): void {
    for (const uri of Array.isArray(allUris) ? allUris : [mainUri]) {
      showPreview(this.previewManager, uri, {
        sideBySide: false,
      });
    }
  }
}

export class ShowPreviewToSideCommand implements Command {
  readonly id: string = "veecleTelemetry.showPreviewToSide";

  constructor(private readonly previewManager: VeecleTelemetryPreviewManager) {}

  execute(uri?: vscode.Uri): void {
    showPreview(this.previewManager, uri, {
      sideBySide: true,
    });
  }
}
