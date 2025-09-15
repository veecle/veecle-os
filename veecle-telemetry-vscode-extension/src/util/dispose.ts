/*
 * Copied / adapted from `vscode`.
 *
 * See License.vscode.txt in the package directory for license information.
 */

import type * as vscode from "vscode";

export function disposeAll(disposables: Iterable<vscode.Disposable>) {
  const errors: any[] = [];

  for (const disposable of disposables) {
    try {
      disposable.dispose();
    } catch (e) {
      errors.push(e);
    }
  }

  if (errors.length === 1) {
    throw errors[0];
  }

  if (errors.length > 1) {
    throw new AggregateError(errors, "Encountered errors while disposing of store");
  }
}

export interface IDisposable {
  dispose(): void;
}

export abstract class Disposable {
  private disposed = false;

  protected disposables: vscode.Disposable[] = [];

  public dispose(): any {
    if (this.disposed) {
      return;
    }
    this.disposed = true;
    disposeAll(this.disposables);
  }

  protected register<T extends IDisposable>(value: T): T {
    if (this.disposed) {
      value.dispose();
    } else {
      this.disposables.push(value);
    }
    return value;
  }

  protected get isDisposed() {
    return this.disposed;
  }
}
