# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- A manuscript-sized browser editor with inline highlights, type switches, explanations above the text, and findings that follow the visible passage.
- A manuscript chart with independently selectable sentence length, paragraph length, dialogue share, finding density, and sentence variation; zoom to the visible passage or navigate from a chart node to the text.
- Background analysis with progress and cancellation for text files up to 16 MiB, plus repeated phrases, wordy constructions, dialogue tags, and conservative tense/person observations in the browser.

### Changed

- `bluepencil-node` carries the workspace version instead of its own. A test fails CI when the npm package, its lockfile, the generated `index.js`, or the VS Code extension disagree with `Cargo.toml`, and `release-node.yml` publishes on every release tag. `bluepencil-node@0.2.0` was published from `main` after 0.2.0 and includes the analysis fixes below.

### Fixed

- Manuscript charts use separate aligned tracks with labeled scales, chapter-aware detail, and sentence or paragraph samples when following the visible passage. Explicit detail choices no longer disappear behind a 250-node cap. The editor uses more width on large displays.
- Repetition and rhythm runs now stop at headings. Person-shift runs no longer combine different grammatical persons. Browser filters suppress conversational perception words and sensory descriptions, and large-document results are retrieved throughout the text.
- Mobile import errors remain visible; the editor supports undo and downloading revisions.

## [0.2.0] - 2026-10-25

### Added

- `bluepencil-node`: native Node.js bindings for `bluepencil-core`, published to npm as `bluepencil-node` (`checkStyle`, `findEchoes`, `findRepeats`, `analyzeRhythm`, `sentenceStarters`, `analyzeDialogue`, `sectionArc`, `report`). First working release is `bluepencil-node@0.1.1`; 0.1.0's darwin-arm64 binary never reached the registry.
- A private browser writing desk, served from the project Pages root, with focused findings and a marked passage view.
- Native Scrivener, Final Draft, and Tinderbox import, plus Pages import on macOS with Pages installed. Ulysses import is available but has only synthetic fixture coverage so far.
- OpenAI, OpenRouter, and OpenAI-compatible model providers alongside Anthropic for opt-in model-backed commands.
- A bundled WebAssembly engine for the VS Code extension and a reusable GitHub Action for prose review.
- Revision-aware HTML reports, a measured quality fixture, and documentation for the new formats and workflows.

### Fixed

- The `init` templates now live inside the `bluepencil` crate, so `cargo install bluepencil` builds from the crates.io package. The crates.io 0.1.0 release carries this fix and is otherwise identical to the tagged release.
- Native import edge cases and report cache behavior found in real-file and regression checks.
- Updated the ZIP and TLS dependency chain to versions without the advisories reported by `cargo deny`.

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

[Unreleased]: https://github.com/writerslogic/bluepencil/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/writerslogic/bluepencil/releases/tag/v0.2.0
[0.1.0]: https://github.com/writerslogic/bluepencil/releases/tag/v0.1.0
