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
| `[model]` | `provider`, `model`, `effort`, `api_key_env`, `max_findings`, `timeout_secs`, `fallback`, `base_url`; see [Output](output.md#editors-notes-with-explain), [facts](commands/facts.md), [voice](commands/voice.md), and [scenes](commands/scenes.md) |

Command-line options override config values. `bluepencil config` prints the merged result and the file it came from.

## Model providers

Model-backed features run only when you invoke `--explain`, `facts`, `voice`, or `scenes`. The default provider is `anthropic` with `ANTHROPIC_API_KEY`. To use another provider:

```toml
[model]
provider = "openai"
model = "gpt-5.2"
# Reads OPENAI_API_KEY from the environment.
```

Use `provider = "openrouter"` with an OpenRouter model ID and `OPENROUTER_API_KEY`. For another service exposing an OpenAI-compatible `/chat/completions` endpoint, use `provider = "compatible"`, set `base_url` to the service's `/v1` base, and set `model`. A compatible endpoint has no API key by default; set `api_key_env` if it requires one. Custom remote endpoints must use HTTPS; HTTP is accepted for loopback addresses only. Provider support for strict JSON Schema output varies by model, so choose a model that supports it.

The Anthropic-only `effort` and `fallback` settings do not apply to OpenAI-compatible requests. Keys are read from environment variables, never from this file. The selected provider receives the manuscript text or finding context sent by the command and may charge for the request.
