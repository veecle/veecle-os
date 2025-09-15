import type * as vscode from "vscode";

export function isTraceFile(document: vscode.TextDocument) {
  return document.languageId === "jsonl" && document.fileName.endsWith(".trace.jsonl");
}
