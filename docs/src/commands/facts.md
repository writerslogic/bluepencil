# facts

A continuity ledger. The model reads the whole manuscript, records every concrete claim about a character, place, or object, and reports the places where those claims disagree: eye color that changes between chapters, a sister who becomes a cousin, a knife that was lost in chapter 3 and drawn in chapter 9.

```sh
bluepencil facts chapters/*.md
bluepencil facts --ledger
```

Each reported fact is a quote the model cited. bluepencil then finds that quote in your files and reports its real `file:line:column`, so a position you see is one you can jump to. A quote the model misremembered is marked `quote not found verbatim`, and its cited line is shown instead.

```text
Mara / eye color
  ch01.md:12:18  grey
      "her grey eyes"
  ch07.md:88:3  brown
      "Mara's brown eyes caught the light"
  → The eye color changes between chapters 1 and 7 with nothing in the text to account for it.

1 contradictions across 42 facts (use --ledger to list every fact)
```

## Options

- `--ledger` prints every fact, sorted by entity and attribute, after the contradictions.

Add `--json` for machine-readable output; each fact carries `verified`, and each contradiction indexes into `facts`.

This command sends the manuscript to the Claude API and needs an API key; see `[model]` in [Configuration](../configuration.md). Results are the model's reading and vary between runs, so `check` does not use them.
