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
- `--no-cache` skip reading the per-file analysis cache (still refreshes it for next time)

Add `--json` for machine-readable output.

## Caching

`report` caches each file's computed metrics in a `.bluepencil-cache/` directory next to
`bluepencil.toml` (or the current directory, if there's no config file), keyed on the file's
content and the config settings that affect analysis. An unchanged file is served from the
cache instead of reanalyzed; a changed file, or a config change, invalidates it automatically.
The directory is gitignored on creation and safe to delete at any time.
