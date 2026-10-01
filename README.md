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
  <a href="https://crates.io/crates/bluepencil"><img src="https://img.shields.io/crates/v/bluepencil.svg?style=flat-square&labelColor=20232a&color=007ec6" alt="crates.io"></a>
  <a href="https://docs.rs/bluepencil-core"><img src="https://img.shields.io/docsrs/bluepencil-core?style=flat-square&labelColor=20232a&color=007ec6" alt="docs.rs"></a>
</p>
<!-- repo-header:end -->

<p align="center">
  <a href="#install">Install</a> &middot;
  <a href="#quick-start">Quick start</a> &middot;
  <a href="#what-it-measures">What it measures</a> &middot;
  <a href="#what-it-reads">What it reads</a> &middot;
  <a href="#in-your-workflow">In your workflow</a> &middot;
  <a href="https://writerslogic.github.io/bluepencil/">Guide</a> &middot;
  <a href="#contributing">Contributing</a>
</p>

---

bluepencil measures prose. It will not tell you whether a paragraph is good. It tells you where to look: a word that echoes eight words later, five sentences in a row that open with "She", a chapter twice as long as its neighbours, a scene with no dialogue in it, a passage that slips from past tense into present. Every result is a `file:line:column` your editor can jump to, every command has a `--json` form, and all of it runs locally in one binary with no account and no network.

Twenty-seven analyses are deterministic and instant. Three more, and an `--explain` flag, hand the measured evidence to a language model for the judgments a word list cannot make: whether a character still sounds like themselves in chapter nine, whether the eye colour established in chapter one survives to the end, whether a scene earns its length. Those are opt-in and clearly marked; nothing else depends on them.

> **You give it:** a folder of chapters in Markdown.
> **It gives you back:** 11.6 hedges per thousand words with each one located, "gravel" repeated four words apart on line 18, three chapters that open on the same word, a Flesch-Kincaid grade per section, and an exit code your CI can gate on.

It understands Markdown, Fountain screenplays, plain text, and Word (`.docx`), handles curly quotes and abbreviations correctly, and treats dialogue as dialogue.

## Install

```sh
brew install writerslogic/tap/bluepencil
```

```sh
cargo install bluepencil
```

Prebuilt binaries for macOS, Linux, and Windows are on the [releases page](https://github.com/writerslogic/bluepencil/releases), with shell and PowerShell installers. A [VS Code extension](editors/vscode/) and a language server (`bluepencil-lsp`) show findings inline; build the extension into a `.vsix` with `npx @vscode/vsce package` in `editors/vscode` and install it with `code --install-extension`. See [Editors](https://writerslogic.github.io/bluepencil/editors.html).

## Quick start

```sh
cd my-novel
bluepencil init                   # writes bluepencil.toml
bluepencil report                 # everything at a glance
bluepencil echoes chapters/03.md  # one analysis, one file
bluepencil report --html report.html
```

```text
$ bluepencil hedges
chapters/01-arrival.md:8:39  hedge  just
chapters/01-arrival.md:12:27  hedge  really
chapters/02-the-letter.md:9:1  hedge  suddenly
    Suddenly

4 found, 11.6 per 1,000 words
```

Pass files or globs, pipe text with `-`, or set `project.files` in `bluepencil.toml` and run commands with no arguments. Every command accepts `--json`; list commands accept `--summary` to group results by word; ranked lists honour `-n`.

## What it measures

<details open>
<summary><strong>Repetition</strong> -- the things a reader notices before you do</summary>

| Command | What it shows |
| --- | --- |
| `echoes` | The same word reappearing within a window (default 50 words) |
| `repeats` | Repeated phrases of three to six words |
| `starters` | Sentence and paragraph openers, and runs of the same opener |
| `tics` | Your own crutch words, from `bluepencil.toml` |
| `overused` | Words used far more than in general English |
| `freq`, `hapax`, `unique` | Word frequency, words used once, lexical diversity (TTR and MATTR) |

</details>

<details open>
<summary><strong>Style</strong> -- word-level habits, each one located</summary>

| Command | What it shows |
| --- | --- |
| `filter` | Filter words that distance the reader (felt, saw, noticed, realized) |
| `hedges` | Hedges and intensifiers (just, really, very) |
| `adverbs` | `-ly` adverbs |
| `cliches` | Stock phrases |
| `passive` | Passive voice (be-verb + past participle) |
| `nominal` | Nominalizations ("make a decision" where a verb would be stronger) |
| `tags` | Dialogue tags, split into plain, adverb-modified, and showy |

</details>

<details open>
<summary><strong>Shape</strong> -- rhythm, structure, and readability per section</summary>

| Command | What it shows |
| --- | --- |
| `count` | Words, characters, sentences, paragraphs, reading time |
| `outline` | Heading tree with word counts |
| `histogram` | Sentence and paragraph length distributions |
| `rhythm` | Sentence length flow, monotonous runs, overlong sentences |
| `readability` | Flesch, Flesch-Kincaid, Gunning Fog, Coleman-Liau, ARI |
| `dialogue` | Dialogue versus narration |
| `arc` | Pacing profile per section: length, dialogue ratio, sentence rhythm, style density |

</details>

<details open>
<summary><strong>Narrative</strong> -- who is speaking, when, and from where</summary>

| Command | What it shows |
| --- | --- |
| `cast` | Character name mentions per section |
| `continuity` | Names that may be the same person spelled inconsistently |
| `speakers` | Who speaks and how much, where attribution can be inferred |
| `tense` | Passages that drift from the dominant tense |
| `pov` | Passages that drift from the dominant point of view |

</details>

<details open>
<summary><strong>With a model</strong> -- editor's judgment, grounded in the measurements above</summary>

| Command | What it shows |
| --- | --- |
| `facts` | Continuity ledger: what the text states about each character, place, and object, and where it disagrees with itself |
| `voice` | Each speaking character's voice as the page shows it, and lines that do not sound like them |
| `scenes` | Scene audit: goal, conflict, and change per scene, with a keep, tighten, cut, or merge verdict |
| `--explain` | On any word-list command or `echoes`: a fix, consider, or keep verdict and a one-line note per finding |

These send your text to the Claude API and need an API key in `ANTHROPIC_API_KEY`. Every claim comes back as a quote, which bluepencil then finds in your files and reports at its real `file:line:column`; a quote it cannot find verbatim is marked. The deterministic core supplies the evidence the model reads: the numbered text, the pacing profile from `arc`, the sentence around each finding. Results vary between runs, so `check` never uses them. See `[model]` in [Configuration](https://writerslogic.github.io/bluepencil/configuration.html).

</details>

<details>
<summary><strong>Files and history</strong></summary>

| Command | What it does |
| --- | --- |
| `report` | All the measurements in one summary, as text, JSON, or HTML |
| `check` | Exit nonzero when configured limits are exceeded |
| `wdiff` | Word-level diff between two versions of a text |
| `progress` | Word count change per file since a git commit or date |
| `split`, `join` | One file per heading, and back again |
| `init`, `config` | Write a starter `bluepencil.toml`; show the effective configuration and where it came from |
| `completions`, `schema` | Shell completions; JSON Schema for a command's `--json` output |

</details>

## What it reads

| Format | How |
| --- | --- |
| Markdown | Headings become sections; emphasis, links, and code are stripped with offsets preserved, so positions point at your source |
| Fountain | Scene headings become sections; dialogue is read from character cues, so action lines and speech are never confused |
| Plain text | Blank-line paragraphs |
| Word `.docx` | Paragraph text and heading styles are extracted and read as Markdown |

Sentences split correctly around abbreviations, initials, ellipses, and dialogue punctuation. Curly and straight quotes both mark dialogue. Force a format with `--as`.

## In your workflow

**Editor.** Any editor that understands `file:line:column` can jump to a result. In Vim, `:cexpr system('bluepencil echoes')` fills the quickfix list. The VS Code extension shows findings as diagnostics, live through the language server or on save through the CLI.

**CI and hooks.** Put limits under `[check]` in `bluepencil.toml` and run `bluepencil check` in a pre-commit hook or a workflow. It exits `1` when an `error`-severity limit is exceeded and `2` on errors; downgrade a rule to `warn` under `[check.severity]` to report it without failing. `check --since <git-ref>` scopes the located rules to lines changed since that ref, rated against the word count of just those lines, so a long manuscript can gate a pull request without flagging what was already there.

**Scripts.** `--json` on every command and `bluepencil schema <command>` for the shape. `report --json` across revisions, diffed, is a revision history of the prose itself.

## Guides

- **[Quick start](https://writerslogic.github.io/bluepencil/quickstart.html)** -- a first run on a manuscript
- **[Configuration](https://writerslogic.github.io/bluepencil/configuration.html)** -- `bluepencil.toml`, word lists, and the model settings
- **[Thresholds and CI](https://writerslogic.github.io/bluepencil/thresholds.html)** -- gating a manuscript repository
- **[Commands](https://writerslogic.github.io/bluepencil/commands/count.html)** -- one page per command with sample output
- **[Contributing](CONTRIBUTING.md)** -- layout, adding a command, tests

## Requirements

- One binary. Nothing else to install.
- Rust 1.88+ to build from source.
- The three model-backed commands and `--explain` need an API key; everything else runs offline.

## Development

```sh
git clone https://github.com/writerslogic/bluepencil.git
cd bluepencil
just check     # fmt, clippy, tests
just demo      # the report on the sample novel in examples/
just audit     # cargo deny
```

The workspace is `bluepencil-core` (parsing, segmentation, statistics, and every analysis; no I/O), `bluepencil` (the CLI), and `bluepencil-lsp` (the language server). Word lists live in `crates/bluepencil-core/data` and are embedded at compile time.

## Why bluepencil

The commercial tools in this space are cloud suites with a GUI: your manuscript leaves your machine, the results are not scriptable, and the price is a subscription. The developer prose linters are scriptable but tuned for documentation, and know nothing about dialogue, point of view, or chapters. bluepencil is the gap between them: a single local binary that understands fiction, reports positions, and fits a shell pipeline or a CI job. When a judgment genuinely needs a reader, it hands a model the measured evidence and pins every answer back to your text, so you get an editor's note with a line number rather than a paragraph of opinion.

## Contributing

Contributions of all sizes are welcome. Check the [issue tracker](https://github.com/writerslogic/bluepencil/issues) or see [CONTRIBUTING.md](CONTRIBUTING.md) for setup and conventions.

**Areas where help is especially welcome:**
- Word lists and sentence-splitting rules for languages other than English
- Real-manuscript false positives for `tense`, `pov`, `speakers`, and `continuity`
- Editor integrations beyond VS Code

## Security

Found a vulnerability? Please report it privately; see [SECURITY.md](SECURITY.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. &copy; [WritersLogic, Inc.](https://github.com/writerslogic)

<p align="center">
  <a href="https://github.com/writerslogic/bluepencil">GitHub</a> &middot;
  <a href="https://crates.io/crates/bluepencil">crates.io</a> &middot;
  <a href="https://writerslogic.github.io/bluepencil/">Guide</a> &middot;
  <a href="https://github.com/writerslogic/bluepencil/issues">Issues</a> &middot;
  <a href="CHANGELOG.md">Changelog</a>
</p>
