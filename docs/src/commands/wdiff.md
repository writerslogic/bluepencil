# wdiff

A word-level diff between two versions of the same text, with net words added and removed. Either argument can be `-` to read stdin, but not both.

```sh
bluepencil wdiff draft-1.md draft-2.md
```

Added words are wrapped in `{+...+}`, removed words in `[-...-]`.

Add `--json` for machine-readable output.
