import * as fs from "node:fs";
import * as path from "node:path";
import { spawn } from "node:child_process";
import * as vscode from "vscode";
import { registerCompletions } from "./completions";

type Severity = "error" | "warning" | "info";

interface DiagnosticItem {
  severity: Severity;
  code: string;
  path: string;
  line: number;
  column: number;
  message: string;
  hint?: string;
}

interface CompileResult {
  valid: boolean;
  diagnostics: DiagnosticItem[];
}

const collection = vscode.languages.createDiagnosticCollection("siammock");
const pending = new Map<string, NodeJS.Timeout>();

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(collection);
  registerCompletions(context);

  const scheduleIfEnabled = (document: vscode.TextDocument) => {
    if (!isSiamMockDocument(document)) {
      return;
    }
    const config = vscode.workspace.getConfiguration("siammock");
    if (!config.get<boolean>("validateOnChange")) {
      return;
    }
    scheduleValidate(document);
  };

  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => {
      if (isSiamMockDocument(document)) {
        void validateDocument(document);
      }
    }),
    vscode.workspace.onDidChangeTextDocument((event) => scheduleIfEnabled(event.document)),
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (isSiamMockDocument(document)) {
        void validateDocument(document);
      }
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      collection.delete(document.uri);
      pending.delete(document.uri.toString());
    }),
    vscode.commands.registerCommand("siammock.validateActiveFile", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || !isSiamMockDocument(editor.document)) {
        void vscode.window.showInformationMessage(
          "Open a .jsonsi file or a mock/*.json config file first."
        );
        return;
      }
      void validateDocument(editor.document);
    })
  );

  for (const document of vscode.workspace.textDocuments) {
    if (isSiamMockDocument(document)) {
      void validateDocument(document);
    }
  }
}

export function deactivate(): void {
  collection.clear();
  for (const timer of pending.values()) {
    clearTimeout(timer);
  }
  pending.clear();
}

function isSiamMockDocument(document: vscode.TextDocument): boolean {
  if (document.languageId === "jsonsi") {
    return true;
  }

  if (document.languageId !== "json") {
    return false;
  }

  const config = vscode.workspace.getConfiguration("siammock");
  const glob = config.get<string>("mockJsonGlob") ?? "mock/*";
  const relative = vscode.workspace.asRelativePath(document.uri, false);

  return matchGlob(relative.replace(/\\/g, "/"), glob);
}

function matchGlob(relativePath: string, pattern: string): boolean {
  const regex = new RegExp(
    "^" +
      pattern
        .replace(/[.+^${}()|[\]\\]/g, "\\$&")
        .replace(/\*\*/g, "§§")
        .replace(/\*/g, "[^/]*")
        .replace(/§§/g, ".*") +
      "$"
  );
  return regex.test(relativePath);
}

function scheduleValidate(document: vscode.TextDocument): void {
  const key = document.uri.toString();
  const existing = pending.get(key);
  if (existing) {
    clearTimeout(existing);
  }

  const config = vscode.workspace.getConfiguration("siammock");
  const debounce = config.get<number>("validateDebounceMs") ?? 400;

  const timer = setTimeout(() => {
    pending.delete(key);
    void validateDocument(document);
  }, debounce);

  pending.set(key, timer);
}

async function validateDocument(document: vscode.TextDocument): Promise<void> {
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
  if (!workspaceFolder) {
    return;
  }

  const binary = resolveBinary(workspaceFolder);
  const relativePath = vscode.workspace.asRelativePath(document.uri, false);
  const result = await runValidate(binary, document.getText(), relativePath, workspaceFolder.uri.fsPath);

  if (!result) {
    collection.set(document.uri, [
      new vscode.Diagnostic(
        new vscode.Range(0, 0, 0, 1),
        "Could not run SiamMock validator. Build with: cargo build",
        vscode.DiagnosticSeverity.Error
      ),
    ]);
    return;
  }

  const diagnostics = result.diagnostics.map((item) => toVsCodeDiagnostic(item, document));
  collection.set(document.uri, diagnostics);
}

function resolveBinary(folder: vscode.WorkspaceFolder): string {
  const config = vscode.workspace.getConfiguration("siammock");
  const configured = config.get<string>("binaryPath") ?? "";

  if (configured && !configured.includes("${workspaceFolder}")) {
    return configured;
  }

  const root = folder.uri.fsPath;
  const resolvedConfig = configured.replace("${workspaceFolder}", root);

  if (resolvedConfig && fs.existsSync(resolvedConfig)) {
    return resolvedConfig;
  }

  const exe = process.platform === "win32" ? "SiamMock.exe" : "SiamMock";
  const debug = path.join(root, "target", "debug", exe);
  if (fs.existsSync(debug)) {
    return debug;
  }

  const release = path.join(root, "target", "release", exe);
  if (fs.existsSync(release)) {
    return release;
  }

  return exe;
}

function runValidate(
  binary: string,
  source: string,
  fileLabel: string,
  cwd: string
): Promise<CompileResult | null> {
  return new Promise((resolve) => {
    const child = spawn(
      binary,
      ["validate", "--stdin", "--json", "--file", fileLabel],
      { cwd, stdio: ["pipe", "pipe", "pipe"] }
    );

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk: Buffer) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk: Buffer) => {
      stderr += chunk.toString();
    });

    child.on("error", () => resolve(null));
    child.on("close", (code) => {
      if (!stdout.trim()) {
        if (stderr) {
          console.error("[siammock-editor]", stderr);
        }
        resolve(null);
        return;
      }

      try {
        resolve(JSON.parse(stdout) as CompileResult);
      } catch {
        console.error("[siammock-editor] invalid JSON from validator", stdout, stderr, code);
        resolve(null);
      }
    });

    child.stdin.write(source);
    child.stdin.end();
  });
}

function toVsCodeDiagnostic(item: DiagnosticItem, document: vscode.TextDocument): vscode.Diagnostic {
  const line = Math.max(0, item.line - 1);
  const column = Math.max(0, item.column - 1);
  const lineText = document.lineAt(line).text;
  const endColumn = Math.min(lineText.length, column + 1);

  const range = new vscode.Range(line, column, line, Math.max(endColumn, column + 1));
  const severity =
    item.severity === "warning"
      ? vscode.DiagnosticSeverity.Warning
      : item.severity === "info"
        ? vscode.DiagnosticSeverity.Information
        : vscode.DiagnosticSeverity.Error;

  const diagnostic = new vscode.Diagnostic(range, formatMessage(item), severity);
  diagnostic.code = item.code;
  diagnostic.source = "SiamMock";
  return diagnostic;
}

function formatMessage(item: DiagnosticItem): string {
  if (item.hint) {
    return `${item.message}\n→ ${item.hint}`;
  }
  return item.message;
}
