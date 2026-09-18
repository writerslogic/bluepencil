# report

Every metric in one summary, with a per-file table when there are several inputs.

```sh
bluepencil report --html report.html
```

## Options

- `--html PATH` also write a standalone HTML report
- `--trend-since GIT_REF` add a word-count and style-flag trend chart to the HTML report,
  sampled from git history since this commit, tag, or date (requires `--html`)
- `--trend-points N` number of points to sample between `--trend-since` and `HEAD` (default 8)

Add `--json` for machine-readable output.
