# Contributing

Thanks for helping make bluepencil better.

## Setup

Install Rust (the toolchain is pinned in `rust-toolchain.toml`) and optionally [just](https://github.com/casey/just).

```sh
just check    # fmt, clippy, tests
just demo     # run the report on the sample novel
```

## Layout

- `crates/bluepencil-core` holds parsing, tokenizing, sentence splitting, statistics, and every analysis. It has no I/O.
- `crates/bluepencil` is the CLI: argument parsing, config, file loading, and output.
- `crates/bluepencil-core/data` holds the bundled word lists, embedded at compile time.

## Adding a command

1. Put the analysis in `crates/bluepencil-core/src/analysis/` and return plain data or `Finding`s with spans.
2. Add the subcommand to `cli.rs` and a module in `crates/bluepencil/src/commands/`.
3. Support `--json`.
4. Add a page under `docs/src/commands/` and a row to the README table.

## Tests

Add tests for logic that is genuinely tricky, such as tokenizing and sentence boundaries. Keep them focused.

## Word lists

Suggestions for the bundled lists are welcome. One entry per line, lowercase, and please keep additions widely applicable rather than genre-specific.
