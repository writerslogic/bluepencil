# schema

Prints the JSON Schema for a command's `--json` output.

```sh
bluepencil schema count
```

Only commands migrated onto typed output structs are covered; the rest still produce valid
JSON, they just don't have a schema yet. Currently covered: `count`.
