# Thresholds and CI

`bluepencil check` measures each file and compares it to the limits under `[check]`. It prints each violation and exits with status `1` if there are any, `0` if all pass, and `2` on errors.

| Key | Meaning |
| --- | --- |
| `max_echoes_per_1k` | Echoes per 1,000 words |
| `max_adverbs_per_1k` | -ly adverbs per 1,000 words |
| `max_filter_per_1k` | Filter words per 1,000 words |
| `max_hedges_per_1k` | Hedges per 1,000 words |
| `max_tics_per_1k` | Tics per 1,000 words |
| `max_cliches` | Total cliches |
| `max_repeated_starter_runs` | Runs of sentences with the same opener |
| `max_monotonous_runs` | Runs of similar-length sentences |
| `max_sentence_words` | Longest sentence |
| `min_mattr` | Minimum lexical diversity |
| `max_grade` | Flesch-Kincaid grade |
| `min_dialogue_ratio`, `max_dialogue_ratio` | Share of words in dialogue, from 0 to 1 |
