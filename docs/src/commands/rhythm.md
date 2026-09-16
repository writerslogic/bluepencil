# rhythm

A sparkline of sentence lengths, flagging runs of similar-length sentences and sentences that run long.

```sh
bluepencil rhythm --bars chapter.md
```

## Options

- `--run N`, `--tolerance N`, `--long N` override `[rhythm]` settings
- `--bars` prints one bar per sentence with its line number

Add `--json` for machine-readable output.
