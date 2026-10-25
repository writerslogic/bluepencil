# bluepencil for VS Code

A local prose analysis companion for Markdown, Fountain, and plain text. The extension includes the bluepencil Rust analysis engine as WebAssembly, so installing the extension is enough to see observations while you write. Your text stays in the editor.

The bundled engine checks open files after a short pause and shows repeated words, familiar sentence openers, clichés, filter words, hedges, adverbs, passive constructions, sentence rhythm, and long sentences in the Problems panel. Select a diagnostic to jump to the passage. The **Check current file** command runs it immediately; **Run report** shows counts and up to 25 observations in the output panel. Files up to 1 MiB are supported by the bundled engine.

Optional integrations remain available:

| Setting | Default | Use |
| --- | --- | --- |
| `bluepencil.lsp.enable` | `false` | Run an installed `bluepencil-lsp` instead of bundled diagnostics. |
| `bluepencil.lsp.path` | `bluepencil-lsp` | Path to the external language server. |
| `bluepencil.cli.enable` | `false` | Use an installed CLI for configured checks and full reports. |
| `bluepencil.cli.path` | `bluepencil` | Path to the external CLI. |
| `bluepencil.cli.commands` | adverbs, passive, cliches, filter, hedges, tics | CLI commands used on save. |

The external CLI supports native writing app files and project configuration. The bundled editor engine analyzes the open text and does not require a binary on your `PATH`.

## Build a VSIX

Install Rust, the `wasm32-unknown-unknown` target, wasm-pack, and Node.js. Then run `npm ci` and `npx @vscode/vsce package` from this directory. The prepublish step builds the WebAssembly engine and compiles the extension. The resulting VSIX includes the engine for macOS, Linux, and Windows.

Licensed under MIT or Apache-2.0, at your option.
