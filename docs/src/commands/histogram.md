# histogram

Distribution of sentence lengths, paragraph lengths, or sentences per paragraph as a terminal bar chart.

```sh
bluepencil histogram --of paragraphs --width 10 --cap 200
```

## Options

- `--of sentences|paragraphs|paragraph-sentences`
- `--width N` bucket width
- `--cap N` values at or above this share the last bucket

Add `--json` for machine-readable output.
