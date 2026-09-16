<!-- repo-header:start -->
<img src="https://raw.githubusercontent.com/writerslogic/bluepencil/main/assets/logo.png" alt="bluepencil logo" width="120" align="left">

<h3>bluepencil</h3>

<p><strong>Prose analysis for writers: echoes, repeats, rhythm, dialogue, readability, and more</strong></p>

<br clear="left">

<p align="center">
  <a href="https://github.com/writerslogic/bluepencil/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/writerslogic/bluepencil/ci.yml?style=flat-square&labelColor=20232a&branch=main&label=CI" alt="CI"></a>
  <a href="https://securityscorecards.dev/viewer/?uri=github.com/writerslogic/bluepencil"><img src="https://img.shields.io/ossf-scorecard/github.com/writerslogic/bluepencil?style=flat-square&labelColor=20232a&label=OpenSSF" alt="OpenSSF Scorecard"></a>
  <a href=".bestpractices.json"><img src="https://img.shields.io/badge/best%20practices-evidence%20reviewed-6a4c93?style=flat-square&labelColor=20232a" alt="Best Practices Evidence"></a>
  <a href="https://github.com/writerslogic/bluepencil/blob/main/LICENSE-APACHE"><img src="https://img.shields.io/github/license/writerslogic/bluepencil?style=flat-square&labelColor=20232a&color=007ec6&label=license" alt="License"></a>
  <a href="https://github.com/writerslogic/bluepencil/blob/main/CODE_OF_CONDUCT.md"><img src="https://img.shields.io/badge/code%20of%20conduct-Contributor%20Covenant%202.1-6a4c93?style=flat-square&labelColor=20232a" alt="Code of Conduct"></a>
  <a href="https://github.com/sponsors/dcondrey"><img src="https://img.shields.io/badge/GitHub%20Sponsors-Sponsor-EA4AAA?style=flat-square&labelColor=20232a" alt="GitHub Sponsors"></a>
</p>
<!-- repo-header:end -->

---

It understands Markdown, Fountain screenplays, and plain text, handles curly quotes and abbreviations correctly, and reports `file:line:column` locations your editor can jump to.

## Install

```sh
brew install writerslogic/tap/bluepencil
# or
cargo install bluepencil
```

## Quick start

```sh
cd my-novel
bluepencil init                 # writes bluepencil.toml
bluepencil report               # everything at a glance
bluepencil echoes chapters/03.md
bluepencil report --html report.html
```

Pass files or globs, pipe text with `-`, or set `project.files` in `bluepencil.toml` and run commands with no arguments.

## Commands

| Command | What it shows |
| --- | --- |
| `count` | Words, characters, sentences, paragraphs, reading time |
| `outline` | Heading tree with word counts per section |
| `histogram` | Sentence and paragraph length distributions |
| `rhythm` | Sentence length flow, monotonous runs, overlong sentences |
| `freq` | Word frequency, function words excluded by default |
| `unique` | Lexical diversity (TTR and MATTR) |
| `hapax` | Words used exactly once |
| `repeats` | Repeated phrases |
| `echoes` | The same word reappearing within a window |
| `starters` | Sentence and paragraph openers, runs of the same opener |
| `tics` | Your own crutch words from the config |
| `filter` | Filter words (felt, saw, noticed, realized) |
| `hedges` | Hedges and intensifiers (just, really, very) |
| `adverbs` | `-ly` adverbs |
| `cliches` | Stock phrases |
| `passive` | Passive voice constructions (be-verb + past participle) |
| `overused` | Words used far more than in general English usage |
| `readability` | Flesch, Flesch-Kincaid, Gunning Fog, Coleman-Liau, ARI, per section |
| `dialogue` | Dialogue versus narration, per section |
| `report` | All of the above in one summary, optionally as HTML |
| `wdiff` | Word-level diff between two versions of the same text |
| `split` | Split a manuscript into one file per heading |
| `join` | Concatenate manuscript files into one |
| `check` | Exit nonzero when configured limits are exceeded |
| `init` | Write a starter `bluepencil.toml` |
| `completions` | Shell completions |

Every command accepts `--json`. List commands accept `--summary` to group results by word.

## Continuous integration

Add limits under `[check]` in `bluepencil.toml`, then run `bluepencil check` in a pre-commit hook or CI job. It exits `1` when a limit is exceeded and `2` on errors.

## Roadmap

`nominal`, dialogue `tags` and `speakers`, `cast`, `tense` and `pov` drift, `progress` tracking, `.docx` input, a language server, and a VS Code extension.

## License

Licensed under either of MIT or Apache-2.0, at your option.
