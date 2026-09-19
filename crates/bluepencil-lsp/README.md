# bluepencil-lsp

A Language Server Protocol server for [bluepencil](../../README.md). Runs the
same prose-analysis engine as the CLI (`bluepencil-core`) against files opened
in an editor and publishes findings as diagnostics: echoes, adverbs, clichés,
filter words, hedges, and passive voice.

## Build

```sh
cargo build --release -p bluepencil-lsp
```

The binary is written to `target/release/bluepencil-lsp` (or your configured
`CARGO_TARGET_DIR`).

## Use

The server speaks LSP over stdio. Point any LSP-capable editor at the built
binary, no arguments needed:

```json
{
  "command": "/path/to/target/release/bluepencil-lsp"
}
```

It recognizes `markdown`, `fountain`, and `plaintext`/`text` language IDs
directly; for anything else it falls back to the file extension
(`.md`/`.markdown`/`.mdown`/`.mkd`/`.mdx` -> Markdown, `.fountain`/`.spmd` ->
Fountain, else Plain).

Diagnostics are informational (not errors), tagged with `source: "bluepencil"`
and a `code` equal to the rule name (`adverb`, `cliche`, `filter`, `hedge`,
`passive`, `echo`), so a client can filter or style them per rule.

## Status

v1: diagnostics on open/change/save via full-document sync. No config file
lookup yet (echo detection uses the CLI's own defaults: window 50, min length
4, no name exclusion) and no code actions. See `TODO:` comments in
`src/config.rs` and `src/code_actions.rs`.
