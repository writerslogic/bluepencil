# Output

Human-readable output is the default. Locations are printed as `file:line:column` so terminals and editors can link them.

Every analysis command accepts `--json`. Ranked lists honor `-n/--limit`.

`report --html PATH` writes a self-contained HTML page that works offline and follows your system's light or dark setting.

## Editor's notes with `--explain`

The word-list commands (`tics`, `filter`, `hedges`, `adverbs`, `cliches`, `passive`, `nominal`) and `echoes` accept `--explain`. Each finding is sent to a language model with its surrounding sentence, and comes back with a verdict and a short note:

```text
ch03.md:12:7  hedges  very
    He was very tired by the time the train came.
    → fix: "very tired" is a flat intensifier here; "exhausted" or "asleep on his feet" does the work.
```

Verdicts are `fix`, `consider`, and `keep`. `keep` marks a false positive or a deliberate choice, such as a line of dialogue that sounds like a person talking.

`--explain`, `facts`, `voice`, and `scenes` are the model-backed features that send text to the provider configured under `[model]`. Each is off unless explicitly invoked. Remote providers need an API key and may charge for requests. None changes what the deterministic commands find or how `check` scores results. With `--json`, each explained finding carries a `note` object. At most `model.max_findings` findings are explained per run, 40 by default.
