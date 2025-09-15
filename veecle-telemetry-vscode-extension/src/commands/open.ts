import type { Command } from "#commands/command-manager";
import type { VeecleTelemetryAppManager } from "#views/app";

export class OpenCommand implements Command {
  readonly id: string = "veecleTelemetry.open";

  constructor(private readonly appManager: VeecleTelemetryAppManager) {}

  execute(): void {
    this.appManager.openApp();
  }
}
