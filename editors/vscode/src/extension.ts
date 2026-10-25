import * as cp from "node:child_process";
import * as vscode from "vscode";
import { analyze_full } from "../engine/bluepencil_web";
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
const SUPPORTED_EXTENSIONS = [".md", ".markdown", ".mdown", ".mkd", ".mdx", ".fountain", ".spmd", ".txt"];

let client: LanguageClient | undefined;
let output: vscode.OutputChannel;
let diagnostics: vscode.DiagnosticCollection;
let lspUnavailableWarned = false;
const pending = new Map<string, NodeJS.Timeout>();

interface BundledFinding {
    rule: string;
    message: string;
    start: number;
    end: number;
}

interface BundledAnalysis {
    words: number;
    sentences: number;
    findings: BundledFinding[];
    truncated: boolean;
}

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
                void runCheck(doc);
            }
        }),
    );

    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(({ document }) => {
        if (!isSupportedDocument(document) || isLspRunning()) return;
        const key = document.uri.toString();
        const previous = pending.get(key);
        if (previous) clearTimeout(previous);
        pending.set(key, setTimeout(() => {
            pending.delete(key);
            void runCheck(document);
        }, 500));
    }));

    context.subscriptions.push(vscode.workspace.onDidOpenTextDocument((doc) => {
        if (isSupportedDocument(doc) && !isLspRunning()) void runCheck(doc);
    }));

    context.subscriptions.push(
        vscode.workspace.onDidCloseTextDocument((doc) => {
            diagnostics.delete(doc.uri);
            const key = doc.uri.toString();
            const timer = pending.get(key);
            if (timer) clearTimeout(timer);
            pending.delete(key);
        }),
    );

    void startLanguageClient(context);
    for (const doc of vscode.workspace.textDocuments) {
        if (isSupportedDocument(doc)) void runCheck(doc);
    }
}

export async function deactivate(): Promise<void> {
    if (client) {
        await client.stop();
        client = undefined;
    }
    for (const timer of pending.values()) clearTimeout(timer);
    pending.clear();
}

function isLspRunning(): boolean {
    return client !== undefined && client.state === State.Running;
}

function isSupportedDocument(doc: vscode.TextDocument): boolean {
    if (SUPPORTED_LANGUAGES.includes(doc.languageId)) {
        return true;
    }
    if (doc.uri.scheme === "untitled" && doc.languageId === "plaintext") return true;
    const fsPath = doc.uri.fsPath.toLowerCase();
    return SUPPORTED_EXTENSIONS.some((ext) => fsPath.endsWith(ext));
}

async function startLanguageClient(context: vscode.ExtensionContext): Promise<void> {
    const config = vscode.workspace.getConfiguration("bluepencil");
    if (!config.get<boolean>("lsp.enable", false)) {
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
                `bluepencil-lsp not found or failed to launch (looked for "${serverPath}"); using the bundled analysis engine.`,
            );
            void vscode.window.setStatusBarMessage(
                "bluepencil: using bundled analysis",
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
    await runCheck(editor.document);
}

function inputFormat(doc: vscode.TextDocument): string {
    const path = doc.uri.fsPath.toLowerCase();
    if (doc.languageId === "fountain" || path.endsWith(".fountain") || path.endsWith(".spmd")) return "fountain";
    if (path.endsWith(".txt") || doc.languageId === "plaintext") return "plain";
    return "markdown";
}

function bundledAnalysis(doc: vscode.TextDocument): BundledAnalysis {
    return JSON.parse(analyze_full(doc.getText(), inputFormat(doc))) as BundledAnalysis;
}

function runBundledCheck(doc: vscode.TextDocument): void {
    try {
        const analysis = bundledAnalysis(doc);
        diagnostics.set(doc.uri, analysis.findings.map((finding) => {
            const diagnostic = new vscode.Diagnostic(
                new vscode.Range(doc.positionAt(finding.start), doc.positionAt(finding.end)),
                finding.message,
                vscode.DiagnosticSeverity.Information,
            );
            diagnostic.source = "bluepencil";
            diagnostic.code = finding.rule;
            return diagnostic;
        }));
        if (analysis.truncated) output.appendLine(`${doc.uri.fsPath}: showing the first 150 observations.`);
    } catch (error) {
        diagnostics.delete(doc.uri);
        output.appendLine(`bluepencil could not analyze ${doc.uri.fsPath}: ${errorMessage(error)}`);
    }
}

async function runCheck(doc: vscode.TextDocument): Promise<void> {
    if (!isSupportedDocument(doc)) return;
    if (doc.uri.scheme === "file" && vscode.workspace.getConfiguration("bluepencil", doc.uri).get<boolean>("cli.enable", false)) {
        await runCliCheck(doc);
    } else {
        runBundledCheck(doc);
    }
}

async function runCliCheck(doc: vscode.TextDocument): Promise<void> {
    const config = vscode.workspace.getConfiguration("bluepencil", doc.uri);
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
                allDiagnostics.push(toDiagnostic(doc, finding, command));
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

function toDiagnostic(doc: vscode.TextDocument, finding: LocatedFinding, command: string): vscode.Diagnostic {
    const position = (line: number, column: number): vscode.Position => {
        const lineIndex = Math.min(Math.max(0, line - 1), doc.lineCount - 1);
        const prefix = [...doc.lineAt(lineIndex).text].slice(0, Math.max(0, column - 1)).join("");
        return new vscode.Position(lineIndex, prefix.length);
    };
    const start = position(finding.line, finding.column);
    const end = position(finding.end_line, finding.end_column);
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
    if (!config.get<boolean>("cli.enable", false)) {
        try {
            const analysis = bundledAnalysis(editor.document);
            output.clear();
            output.appendLine(`bluepencil · ${editor.document.uri.fsPath}`);
            output.appendLine(`${analysis.words} words · ${analysis.sentences} sentences · ${analysis.findings.length} observations`);
            output.appendLine("");
            for (const finding of analysis.findings.slice(0, 25)) {
                const pos = editor.document.positionAt(finding.start);
                output.appendLine(`${pos.line + 1}:${pos.character + 1}  ${finding.rule}  ${finding.message}`);
            }
            if (analysis.truncated) output.appendLine("Showing the first 150 observations; use the CLI for the complete manuscript.");
            output.show(true);
        } catch (error) {
            void vscode.window.showErrorMessage(`bluepencil report failed: ${errorMessage(error)}`);
        }
        return;
    }
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
