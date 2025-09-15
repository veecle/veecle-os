/**
 * This is the entrypoint for the extension.
 *
 * This runs for both the native extension host and the browser extension host.
 */

import * as vscode from "vscode";

import { CommandManager } from "#commands/command-manager";
import { OpenCommand } from "#commands/open";
import { ShowPreviewCommand, ShowPreviewToSideCommand } from "#commands/show-preview";
import { ShowSourceCommand } from "#commands/show-source";
import { isTraceFile } from "#util/file";
import { VeecleTelemetryAppManager } from "#views/app";
import { VeecleTelemetryPreviewManager } from "#views/preview-manager";

export function activate(context: vscode.ExtensionContext) {
  console.log('"veecleTelemetry" shared activation!');

  const appManager = new VeecleTelemetryAppManager(context);
  const webviewManager = new VeecleTelemetryPreviewManager(context);

  context.subscriptions.push(registerCommands(appManager, webviewManager));

  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      // Only allow previewing normal text editors which have a viewColumn: See #101514
      if (typeof editor?.viewColumn === "undefined") {
        return;
      }

      vscode.commands.executeCommand("setContext", "veecleTelemetry.isTraceFile", isTraceFile(editor.document));
    }),
  );
}

function registerCommands(appManager: VeecleTelemetryAppManager, webviewManager: VeecleTelemetryPreviewManager) {
  const commandManager = new CommandManager();

  commandManager.register(new OpenCommand(appManager));
  commandManager.register(new ShowPreviewCommand(webviewManager));
  commandManager.register(new ShowPreviewToSideCommand(webviewManager));
  commandManager.register(new ShowSourceCommand(webviewManager));

  return vscode.Disposable.from(appManager, commandManager);
}
