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
- `crates/bluepencil/templates` holds the `init` templates. The genre presets are mirrored in `examples/<genre>/bluepencil.toml`; a test keeps the two identical.

## Adding a command

1. Put the analysis in `crates/bluepencil-core/src/analysis/` and return plain data or `Finding`s with spans.
2. Add the subcommand to `cli.rs` and a module in `crates/bluepencil/src/commands/`.
3. Support `--json`.
4. Add a page under `docs/src/commands/` and a row to the README table.

## Tests

Add tests for logic that is genuinely tricky, such as tokenizing and sentence boundaries. Keep them focused.

## Browser writing desk

The browser uses the Rust WebAssembly engine and a bundled CodeMirror editor. The build versions the stylesheet, application, worker, and engine assets together so cached files cannot mix incompatible builds. Install the browser dependencies before building the site, then run its integration checks:

```sh
npm --prefix docs/playground ci
bash scripts/build-pages.sh
cd docs/playground
npx playwright install chromium
npm test
```

The checks serve the built site locally and exercise large documents, inline highlights, scrolling, charts, cancellation, editing, and import errors. To check WebKit as well, run `npx playwright install webkit` and `BLUEPENCIL_BROWSER=webkit npm test` from `docs/playground`. Rust rule and chart-value regression tests run with `cargo test -p bluepencil-web -p bluepencil-core` from the repository root.

## Fuzzing

The parsers (`crates/bluepencil-core/src/parse/`) handle arbitrary user files, so they're
fuzzed with `cargo-fuzz`:

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run parse
```

## Word lists

Suggestions for the bundled lists are welcome. One entry per line, lowercase, and please keep additions widely applicable rather than genre-specific.

`data/english_frequency.tsv` (the baseline `overused` compares against) is generated,
not hand-curated: it's the top words by total occurrence in the Google Books Ngram
Viewer 1-gram English dataset (2012-07-01), freely usable per
[Google's own terms](https://books.google.com/ngrams/info) (no share-alike, attribution
appreciated but not required). Regenerate with:

```sh
uv run python scripts/build_english_frequency.py
```
