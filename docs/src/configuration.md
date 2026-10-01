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
| `[model]` | `model`, `effort`, `api_key_env`, `max_findings`, `timeout_secs`, `fallback`, `base_url`; see [Output](output.md#editors-notes-with-explain), [facts](commands/facts.md), [voice](commands/voice.md), and [scenes](commands/scenes.md) |

Command-line options override config values. `bluepencil config` prints the merged result and the file it came from.
