import * as vscode from "vscode";

const COOLDOWN_URL = "http://127.0.0.1:9876/event";

function postEvent(payload: Record<string, unknown>): void {
  fetch(COOLDOWN_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  }).catch(() => {});
}

function classifyDiagnostics(diags: vscode.Diagnostic[]): string {
  const src = diags[0]?.source?.toLowerCase() ?? "";
  const msg = diags.map((d) => d.message.toLowerCase()).join(" ");
  if (src.includes("eslint") || src.includes("tslint") || msg.includes("lint")) {
    return "lint_error";
  }
  if (src.includes("jest") || src.includes("mocha") || msg.includes("test")) {
    return "test_failed";
  }
  return "build_error";
}

export function activate(context: vscode.ExtensionContext): void {
  const diagSub = vscode.languages.onDidChangeDiagnostics((e) => {
    for (const uri of e.uris) {
      const diags = vscode.languages.getDiagnostics(uri);
      const errors = diags.filter((d) => d.severity === vscode.DiagnosticSeverity.Error);
      if (errors.length > 0) {
        postEvent({
          source: "vscode",
          plugin: "vscode",
          event: classifyDiagnostics(errors),
          message: `${errors.length} error(s) in ${uri.fsPath.split(/[/\\]/).pop()}`,
        });
      }
    }
  });
  context.subscriptions.push(diagSub);

  const taskSub = vscode.tasks.onDidEndTaskProcess((e) => {
    if (e.exitCode !== 0) {
      postEvent({
        source: "vscode",
        plugin: "vscode",
        event: "task_failed",
        exit_code: e.exitCode,
      });
    }
  });
  context.subscriptions.push(taskSub);
}

export function deactivate(): void {}
