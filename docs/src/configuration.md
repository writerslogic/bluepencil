# Configuration

bluepencil looks for `bluepencil.toml` in the current directory and each parent directory, using the first one it finds. Pass `--config PATH` to use a specific file. Globs in `project.files` are resolved relative to the config file.

Every setting is optional. Run `bluepencil init` for a commented template.

| Section | Keys |
| --- | --- |
| `[project]` | `files` |
| `[echoes]` | `window`, `min_length`, `include_names`, `ignore` |
| `[repeats]` | `min`, `max`, `count` |
| `[rhythm]` | `run`, `tolerance`, `long` |
| `[starters]` | `run` |
| `[tics]` | `words` |
| `[lexicon]` | `filter`, `hedges`, `cliches`, `stopwords`, `not_adverbs`, `ignore`, `files` |
| `[check]` | see [Thresholds](thresholds.md) |

Command-line options override config values.
