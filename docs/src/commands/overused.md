# overused

Words used far more often than in general English usage, compared against a bundled frequency baseline.

```sh
bluepencil overused --min-ratio 3 --min-count 3
```

## Options

- `--min-ratio N` flag a word once it's used at least this many times more than baseline (default `3.0`)
- `--min-count N` ignore words seen fewer than this many times (default `3`)

Add `--json` for machine-readable output.
