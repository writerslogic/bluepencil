# bluepencil

Prose analysis for writers. bluepencil reads your manuscript and shows you where to look: words that echo too soon, phrases you repeat, sentences that all start the same way, stretches where the rhythm goes flat, and how much of your story is dialogue.

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
| `readability` | Flesch, Flesch-Kincaid, Gunning Fog, Coleman-Liau, ARI, per section |
| `dialogue` | Dialogue versus narration, per section |
| `report` | All of the above in one summary, optionally as HTML |
| `check` | Exit nonzero when configured limits are exceeded |
| `init` | Write a starter `bluepencil.toml` |
| `completions` | Shell completions |

Every command accepts `--json`. List commands accept `--summary` to group results by word.

## Continuous integration

Add limits under `[check]` in `bluepencil.toml`, then run `bluepencil check` in a pre-commit hook or CI job. It exits `1` when a limit is exceeded and `2` on errors.

## Roadmap

`overused` (comparison against English baseline frequencies), `passive`, `nominal`, dialogue `tags` and `speakers`, `cast`, `tense` and `pov` drift, `progress` tracking, `wdiff`, `split` and `join`, `.docx` input, a language server, and a VS Code extension.

## License

Licensed under either of MIT or Apache-2.0, at your option.
