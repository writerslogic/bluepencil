# progress

Word count change per file since a git commit, tag, or date, useful for tracking daily or weekly output.

```sh
bluepencil progress --since HEAD~10
bluepencil progress chapters/*.md --since 2026-09-01 --until HEAD~5
```

## Options

- `--since REF` earlier commit, tag, or date (e.g. `HEAD~10`, `2026-09-01`) to compare from
- `--until REF` later commit to compare to; defaults to the current working tree

Requires running inside a git repository. `--until` bounds both endpoints to history, which lets it report on a file that was renamed or deleted since `--since` as long as every input is a literal path (no globs, no stdin).

Add `--json` for machine-readable output.
