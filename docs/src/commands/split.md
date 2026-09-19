# split

Split a manuscript into one file per heading at the given level.

```sh
bluepencil split manuscript.md --level 1
```

Writes numbered, slugified files (`01-chapter-one.md`, `02-chapter-two.md`, ...) into a directory named after the input file, or the path given with `--out`.

## Options

- `path` the manuscript to split
- `--level N` heading level to split at, `1` = `#`, `2` = `##`, ... (default `1`)
- `--out PATH` directory to write the split files into
