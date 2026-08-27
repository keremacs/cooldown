"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const COOLDOWN_URL = "http://127.0.0.1:9876/event";
function postEvent(payload) {
    fetch(COOLDOWN_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
    }).catch(() => { });
}
function classifyDiagnostics(diags) {
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
function activate(context) {
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
function deactivate() { }
//# sourceMappingURL=extension.js.map