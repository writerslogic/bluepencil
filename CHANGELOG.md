# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `bluepencil-node`: native Node.js bindings for `bluepencil-core`, published to npm as `bluepencil` (`checkStyle`, `findEchoes`, `findRepeats`, `analyzeRhythm`, `sentenceStarters`, `analyzeDialogue`, `sectionArc`, `report`).

### Fixed

- The `init` templates now live inside the `bluepencil` crate, so `cargo install bluepencil` builds from the crates.io package. The crates.io 0.1.0 release carries this fix and is otherwise identical to the tagged release.

## [0.1.0] - 2026-10-01

### Added

- Markdown, Fountain, and plain text parsing with offset-preserving markup removal.
- Sentence splitting that handles abbreviations, initials, ellipses, and dialogue punctuation.
- Commands: `count`, `outline`, `histogram`, `rhythm`, `freq`, `unique`, `hapax`, `repeats`, `echoes`, `starters`, `tics`, `filter`, `hedges`, `adverbs`, `cliches`, `passive`, `overused`, `readability`, `dialogue`, `report`, `check`, `wdiff`, `split`, `join`, `init`, `completions`.
- JSON output for every command and a standalone HTML report.
- `bluepencil.toml` configuration with project globs, custom word lists, and CI thresholds.
- `[check.severity]` to downgrade a threshold to a non-failing warning.
- `bluepencil check --since <git-ref>` to scope location-based rules to changed lines.
- Word (`.docx`) input, a language server (`bluepencil-lsp`), and a VS Code extension.
- Commands: `nominal`, `tags`, `speakers`, `cast`, `tense`, `pov`, `arc`, `progress`, `config`, `schema`.
- Model-backed commands `facts` (continuity ledger), `voice` (character voice consistency), and `scenes` (scene audit), and an `--explain` flag on the word-list commands and `echoes`. Each sends the text to the Claude API, is off unless asked for, and pins every cited quote back to a real `file:line:column`. Configured under `[model]`.

[Unreleased]: https://github.com/writerslogic/bluepencil/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/writerslogic/bluepencil/releases/tag/v0.1.0
