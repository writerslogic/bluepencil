# Editors

Any editor that understands `file:line:column` can jump to results. In Vim, `:cexpr system('bluepencil echoes')` loads echoes into the quickfix list. In VS Code, Cmd-click a location in the integrated terminal.

## VS Code extension

The `bluepencil` VS Code extension (`editors/vscode/`) shows findings inline as diagnostics.
It has two modes:

- **Language server.** If a `bluepencil-lsp` binary is on `PATH` (or set via
  `bluepencil.lsp.path`), the extension connects to it over stdio and diagnostics update live.
- **CLI fallback.** Without a language server, the extension runs the `bluepencil` CLI
  (`bluepencil.cli.path`) on save for Markdown and Fountain files, using the location-carrying
  `--json` subcommands (`adverbs`, `passive`, `cliches`, `filter`, `hedges`, `tics`, and similar
  — configurable via `bluepencil.cli.commands`) and turns each finding into a diagnostic.

Each GitHub release carries the extension as a `.vsix`; install it with `code --install-extension bluepencil-0.1.0.vsix`. See `editors/vscode/README.md` for settings details. A language server
(`crates/bluepencil-lsp`) is what the extension's first mode talks to.
