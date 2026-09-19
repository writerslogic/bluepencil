# bluepencil for VS Code

Prose analysis for writers: echoes, repeats, rhythm, dialogue, readability, and more, inline
in the editor.

This extension has two complementary modes:

1. **Language server (preferred).** If a `bluepencil-lsp` binary is on your `PATH` (or pointed
   to via `bluepencil.lsp.path`), the extension launches it over stdio and shows its
   diagnostics live as you type.
2. **CLI fallback (works today).** If the language server binary isn't found, or
   `bluepencil.lsp.enable` is `false`, the extension shells out to the `bluepencil` CLI on
   every save of a supported file and shows the findings as inline diagnostics.

Both modes cover Markdown (`.md`, `.markdown`) and Fountain (`.fountain`) files. VS Code has no
built-in Fountain language, so this extension registers a minimal `fountain` language ID; if
you already have a dedicated Fountain extension installed, its language contribution takes
precedence and this one's `onStartupFinished` activation event still ensures bluepencil
activates for `.fountain` files either way.

## Requirements

- The [`bluepencil`](https://github.com/writerslogic/bluepencil) CLI installed and on your
  `PATH` (or configured via `bluepencil.cli.path`) for CLI-fallback checks and the
  **bluepencil: Run report** command.
- Optionally, a `bluepencil-lsp` binary on your `PATH` (or configured via `bluepencil.lsp.path`)
  for live, as-you-type diagnostics.

If neither is present, the extension does nothing destructive: it logs a message to the
**bluepencil** output channel once and stays quiet rather than repeatedly erroring.

## Commands

- **bluepencil: Check current file** — runs the configured CLI checks against the active file
  and populates the Problems panel.
- **bluepencil: Run report** — runs `bluepencil report` on the active file and shows the output
  in the **bluepencil** output channel.
- **bluepencil: Restart language server** — stops and relaunches the `bluepencil-lsp` client,
  useful after installing the server or changing `bluepencil.lsp.path`.

## Settings

| Setting | Default | Description |
| --- | --- | --- |
| `bluepencil.lsp.enable` | `true` | Try to launch `bluepencil-lsp` for live diagnostics. |
| `bluepencil.lsp.path` | `"bluepencil-lsp"` | Path to the language server binary. |
| `bluepencil.cli.enable` | `true` | Run CLI-based checks on save when the language server is unavailable. |
| `bluepencil.cli.path` | `"bluepencil"` | Path to the `bluepencil` CLI binary. |
| `bluepencil.cli.commands` | `["adverbs", "passive", "cliches", "filter", "hedges", "tics"]` | `bluepencil` subcommands run on save. Each must support `--json` with a `findings` array carrying `file`/`line`/`column` (the "list" style commands — `adverbs`, `passive`, `cliches`, `filter`, `hedges`, `tics`, `nominal`, and similar). Commands with a different JSON shape, such as `echoes`, aren't supported by the fallback parser and shouldn't be added here. |

## How CLI-fallback diagnostics work

On save of a supported file, the extension runs each configured subcommand with `--json
<file>` and parses the JSON `findings` array (`{ file, line, column, end_line, end_column,
rule, message, text }`). `line`/`column` from the CLI are one-based; VS Code positions are
zero-based, so the extension subtracts one from both before building a `Diagnostic`.

## License

Licensed under either of MIT or Apache-2.0, at your option, matching the rest of the
bluepencil project.
