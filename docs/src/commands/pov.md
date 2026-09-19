# pov

Passages that drift from the document's dominant point of view (first, second, or third person). Dialogue is excluded: a character addressing someone as "you" doesn't change the narrator's person.

```sh
bluepencil pov --run 3
```

Classified by which set of personal pronouns (`I`/`we`, `you`, `he`/`she`/`they`) appears most in a sentence's narration; a sentence with none, or a tie, is left unclassified.

## Options

- `--run N` flag runs of at least this many consecutive sentences in a non-dominant point of view (default `3`)

Add `--json` for machine-readable output.
