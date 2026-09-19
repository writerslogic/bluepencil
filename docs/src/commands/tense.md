# tense

Passages that drift from the document's dominant tense (past or present). Dialogue is excluded: a character speaking in a different tense than the narration is normal, not drift.

```sh
bluepencil tense --run 3
```

No POS tagger: a sentence is classified by past/present auxiliary verbs (`was`, `is`, `had`, `does`, ...) and regular `-ed` inflections, so a present-tense main verb with no auxiliary ("she walks") isn't detected.

## Options

- `--run N` flag runs of at least this many consecutive sentences in the non-dominant tense (default `3`)

Add `--json` for machine-readable output.
