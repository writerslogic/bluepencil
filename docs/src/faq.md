# FAQ

## Why does my word count differ from my word processor?

bluepencil counts hyphenated compounds and contractions as one word, like most word processors, but skips headings, code, and markup. Numbers written as digits are not counted as words.

## Why MATTR instead of unique words divided by total words?

The raw ratio falls as a text gets longer, so a novel always looks less varied than a short story. MATTR averages the ratio over a sliding window, which makes texts of different lengths comparable.

## Does bluepencil send my writing anywhere?

The deterministic commands run locally and make no network requests. `facts`, `voice`, `scenes`, and `--explain` send manuscript text or finding context to the configured model provider when you explicitly run them. Remote providers require an API key and may charge for requests. `check` never invokes a model. See [Configuration](configuration.md) for the model settings.
