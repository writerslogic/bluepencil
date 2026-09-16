# repeats

Repeated phrases of several words. Shorter phrases that only occur inside a longer repeat are folded into it.

```sh
bluepencil repeats --min 2 --max 5 --locations
```

## Options

- `--min`, `--max` phrase length in words
- `--count N` minimum occurrences
- `--locations` list every occurrence

Add `--json` for machine-readable output.
