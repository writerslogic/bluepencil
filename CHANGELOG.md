# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Markdown, Fountain, and plain text parsing with offset-preserving markup removal.
- Sentence splitting that handles abbreviations, initials, ellipses, and dialogue punctuation.
- Commands: `count`, `outline`, `histogram`, `rhythm`, `freq`, `unique`, `hapax`, `repeats`, `echoes`, `starters`, `tics`, `filter`, `hedges`, `adverbs`, `cliches`, `passive`, `overused`, `readability`, `dialogue`, `report`, `check`, `wdiff`, `init`, `completions`.
- JSON output for every command and a standalone HTML report.
- `bluepencil.toml` configuration with project globs, custom word lists, and CI thresholds.
