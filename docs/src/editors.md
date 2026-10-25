# Editors

Any editor that understands `file:line:column` can jump to results. In Vim, `:cexpr system('bluepencil echoes')` loads echoes into the quickfix list.

## VS Code extension

The [bluepencil VS Code extension](https://github.com/writerslogic/bluepencil/tree/main/editors/vscode) includes the Rust analysis engine as WebAssembly. Install its VSIX and open a Markdown, Fountain, or plain-text file: observations appear in the Problems panel as you edit, with no separate binary or account. **Check current file** reruns it immediately; **Run report** shows counts and a short review list.

Download the [0.2.0 VSIX](https://github.com/writerslogic/bluepencil/releases/download/vscode-extension-2026-10-25/bluepencil-0.2.0.vsix), then install it with `code --install-extension bluepencil-0.2.0.vsix`. To build one from source, install Rust, the `wasm32-unknown-unknown` target, wasm-pack, and Node.js, then run `npm ci` and `npx @vscode/vsce package` in `editors/vscode`.

For project configuration and full CLI reports, install `bluepencil` and set `bluepencil.cli.enable` to `true`. To use the external language server, install `bluepencil-lsp` and set `bluepencil.lsp.enable` to `true`. See the [extension README](https://github.com/writerslogic/bluepencil/tree/main/editors/vscode#readme) for settings.
