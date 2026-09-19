# check

Compares each file against the limits in `[check]` and exits with status 1 if any are exceeded, 2 on errors. See [Thresholds](../thresholds.md).

```sh
bluepencil check
bluepencil check --since HEAD~1
```

## Options

- `--since GIT_REF` scope line-scoped rules (echoes, adverbs, filter, hedges, tics, passive,
  cliches, repeated openers, monotonous runs, long sentences) to lines changed since this git
  ref, rated against the word count of just those lines rather than the whole file. Rules that
  can't be meaningfully scoped to a diff (`overused`, `grade`, `mattr`, dialogue ratio) are
  skipped, noted in the output.

Add `--json` for machine-readable output.
