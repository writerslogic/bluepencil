import * as cp from "node:child_process";
import * as vscode from "vscode";
import {
    CloseAction,
    ErrorAction,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    State,
    TransportKind,
} from "vscode-languageclient/node";

const SUPPORTED_LANGUAGES = ["markdown", "fountain"];
const SUPPORTED_EXTENSIONS = [".md", ".markdown", ".fountain"];

let client: LanguageClient | undefined;
let output: vscode.OutputChannel;
let diagnostics: vscode.DiagnosticCollection;
let lspUnavailableWarned = false;

/** bluepencil's `list_command` JSON shape: { total, per_1k_words, summary, findings }. */
interface LocatedFinding {
    file: string;
    line: number;
    column: number;
    end_line: number;
    end_column: number;
    rule: string;
    message: string;
    text: string;
}

interface ListCommandOutput {
    total: number;
    per_1k_words: number;
    summary: unknown;
    findings: LocatedFinding[];
}

export function activate(context: vscode.ExtensionContext): void {
    output = vscode.window.createOutputChannel("bluepencil");
    diagnostics = vscode.languages.createDiagnosticCollection("bluepencil");
    context.subscriptions.push(output, diagnostics);

    context.subscriptions.push(
        vscode.commands.registerCommand("bluepencil.checkFile", () => runCliCheckOnActiveEditor()),
        vscode.commands.registerCommand("bluepencil.runReport", () => runReportCommand()),
        vscode.commands.registerCommand("bluepencil.restartLsp", () => restartLanguageClient(context)),
    );

    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((doc) => {
            if (isSupportedDocument(doc) && !isLspRunning()) {
                void runCliCheck(doc);
            }
        }),
    );

    context.subscriptions.push(
        vscode.workspace.onDidCloseTextDocument((doc) => {
            diagnostics.delete(doc.uri);
        }),
    );

    void startLanguageClient(context);
}

export async function deactivate(): Promise<void> {
    if (client) {
        await client.stop();
        client = undefined;
    }
}

function isLspRunning(): boolean {
    return client !== undefined && client.state === State.Running;
}

function isSupportedDocument(doc: vscode.TextDocument): boolean {
    if (SUPPORTED_LANGUAGES.includes(doc.languageId)) {
        return true;
    }
    const fsPath = doc.uri.fsPath.toLowerCase();
    return SUPPORTED_EXTENSIONS.some((ext) => fsPath.endsWith(ext));
}

async function startLanguageClient(context: vscode.ExtensionContext): Promise<void> {
    const config = vscode.workspace.getConfiguration("bluepencil");
    if (!config.get<boolean>("lsp.enable", true)) {
        output.appendLine("bluepencil.lsp.enable is false; using CLI fallback only.");
        return;
    }

    const serverPath = config.get<string>("lsp.path", "bluepencil-lsp");

    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio },
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: SUPPORTED_LANGUAGES.map((language) => ({ scheme: "file", language })),
        outputChannel: output,
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{md,markdown,fountain}"),
        },
        // The task calls for failing quietly rather than repeatedly erroring when the
        // binary isn't there yet, so stop after the first failure instead of letting the
        // default handler retry-and-notify several times.
        errorHandler: {
            error: () => ({ action: ErrorAction.Shutdown, message: undefined, handled: true }),
            closed: () => ({ action: CloseAction.DoNotRestart, handled: true }),
        },
    };

    client = new LanguageClient("bluepencil", "bluepencil", serverOptions, clientOptions);

    try {
        await client.start();
        output.appendLine(`bluepencil-lsp started (${serverPath}).`);
        diagnostics.clear();
        context.subscriptions.push({ dispose: () => void client?.stop() });
    } catch (err) {
        output.appendLine(`bluepencil-lsp failed to start: ${errorMessage(err)}`);
        client = undefined;
        if (!lspUnavailableWarned) {
            lspUnavailableWarned = true;
            output.appendLine(
                `bluepencil-lsp not found or failed to launch (looked for "${serverPath}"); falling back to CLI-based checks on save.`,
            );
            void vscode.window.setStatusBarMessage(
                "bluepencil: language server not found, using CLI checks on save",
                8000,
            );
        }
    }
}

async function restartLanguageClient(context: vscode.ExtensionContext): Promise<void> {
    if (client) {
        await client.stop();
        client = undefined;
    }
    diagnostics.clear();
    lspUnavailableWarned = false;
    await startLanguageClient(context);
}

async function runCliCheckOnActiveEditor(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        void vscode.window.showWarningMessage("bluepencil: no active editor.");
        return;
    }
    await runCliCheck(editor.document);
}

async function runCliCheck(doc: vscode.TextDocument): Promise<void> {
    const config = vscode.workspace.getConfiguration("bluepencil", doc.uri);
    if (!config.get<boolean>("cli.enable", true)) {
        return;
    }
    if (doc.uri.scheme !== "file") {
        return;
    }

    const cliPath = config.get<string>("cli.path", "bluepencil");
    const commands = config.get<string[]>("cli.commands", [
        "adverbs",
        "passive",
        "cliches",
        "filter",
        "hedges",
        "tics",
    ]);

    const allDiagnostics: vscode.Diagnostic[] = [];
    let anySucceeded = false;
    let lastError: string | undefined;

    for (const command of commands) {
        try {
            const findings = await runListCommand(cliPath, command, doc.uri.fsPath);
            anySucceeded = true;
            for (const finding of findings) {
                allDiagnostics.push(toDiagnostic(finding, command));
            }
        } catch (err) {
            lastError = errorMessage(err);
            output.appendLine(`bluepencil ${command} --json failed: ${lastError}`);
        }
    }

    if (!anySucceeded && lastError) {
        diagnostics.delete(doc.uri);
        void vscode.window.setStatusBarMessage(
            `bluepencil: CLI check failed (${lastError}). Check the bluepencil output channel.`,
            8000,
        );
        return;
    }

    diagnostics.set(doc.uri, allDiagnostics);
}

/**
 * `Located.line`/`.column` from `crates/bluepencil-core/src/location.rs` are documented
 * one-based ("One-based line and column"); VS Code's `Position` is zero-based. Subtract 1
 * on both axes. Columns count characters (not bytes/UTF-16 code units), which matches
 * VS Code's character-indexed Position for all-BMP text; astral characters could disagree,
 * an accepted gap for v1.
 */
function toDiagnostic(finding: LocatedFinding, command: string): vscode.Diagnostic {
    const start = new vscode.Position(Math.max(0, finding.line - 1), Math.max(0, finding.column - 1));
    const end = new vscode.Position(Math.max(0, finding.end_line - 1), Math.max(0, finding.end_column - 1));
    const range = start.isBeforeOrEqual(end) ? new vscode.Range(start, end) : new vscode.Range(start, start);
    const diagnostic = new vscode.Diagnostic(range, finding.message, vscode.DiagnosticSeverity.Information);
    diagnostic.source = `bluepencil (${command})`;
    diagnostic.code = finding.rule;
    return diagnostic;
}

function runListCommand(cliPath: string, command: string, filePath: string): Promise<LocatedFinding[]> {
    return new Promise((resolve, reject) => {
        cp.execFile(
            cliPath,
            [command, "--json", filePath],
            { maxBuffer: 10 * 1024 * 1024 },
            (err, stdout, stderr) => {
                if (err) {
                    reject(new Error(stderr?.trim() || err.message));
                    return;
                }
                try {
                    const parsed = JSON.parse(stdout) as ListCommandOutput;
                    resolve(Array.isArray(parsed.findings) ? parsed.findings : []);
                } catch (parseErr) {
                    reject(new Error(`could not parse JSON output: ${errorMessage(parseErr)}`));
                }
            },
        );
    });
}

async function runReportCommand(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        void vscode.window.showWarningMessage("bluepencil: no active editor.");
        return;
    }
    const config = vscode.workspace.getConfiguration("bluepencil", editor.document.uri);
    const cliPath = config.get<string>("cli.path", "bluepencil");
    const filePath = editor.document.uri.fsPath;

    output.appendLine(`\n$ ${cliPath} report ${filePath}`);
    output.show(true);

    cp.execFile(cliPath, ["report", filePath], { maxBuffer: 10 * 1024 * 1024 }, (err, stdout, stderr) => {
        if (stdout) {
            output.append(stdout);
        }
        if (err) {
            output.appendLine(`bluepencil report failed: ${stderr?.trim() || err.message}`);
            void vscode.window.showErrorMessage(`bluepencil report failed: ${errorMessage(err)}`);
        }
    });
}

function errorMessage(err: unknown): string {
    if (err instanceof Error) {
        return err.message;
    }
    return String(err);
}
