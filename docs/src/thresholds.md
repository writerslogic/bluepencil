# Thresholds and CI

`bluepencil check` measures each file and compares it to the limits under `[check]`. It prints each violation and exits with status `1` if there are any, `0` if all pass, and `2` on errors.

| Key | Meaning |
| --- | --- |
| `max_echoes_per_1k` | Echoes per 1,000 words |
| `max_adverbs_per_1k` | -ly adverbs per 1,000 words |
| `max_filter_per_1k` | Filter words per 1,000 words |
| `max_hedges_per_1k` | Hedges per 1,000 words |
| `max_tics_per_1k` | Tics per 1,000 words |
| `max_passive_per_1k` | Passive voice constructions per 1,000 words |
| `max_nominal_per_1k` | Nominalizations per 1,000 words |
| `max_overused` | Total words flagged as overused |
| `max_cliches` | Total cliches |
| `max_repeated_starter_runs` | Runs of sentences with the same opener |
| `max_monotonous_runs` | Runs of similar-length sentences |
| `max_sentence_words` | Longest sentence |
| `min_mattr` | Minimum lexical diversity |
| `max_grade` | Flesch-Kincaid grade |
| `min_dialogue_ratio`, `max_dialogue_ratio` | Share of words in dialogue, from 0 to 1 |

## Severity

Every rule reports at `error` severity by default, which fails `check`. Override a rule to
`warn` under `[check.severity]` in `bluepencil.toml` to have it printed without failing the run:

```toml
[check.severity]
adverbs = "warn"
```

The key is the rule name (`echoes`, `adverbs`, `filter`, `hedges`, `tics`, `passive`, `cliches`,
`overused`, and so on, matching the `max_*`/`min_*` keys above without their prefix).
