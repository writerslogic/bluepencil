# continuity

Character names that may be the same person spelled inconsistently, clustered across every
input file (so a spelling that drifted between chapter 3 and chapter 20 shows up even though
neither file alone looks wrong).

```sh
bluepencil continuity chapters/*.md
```

Clusters two proper nouns when they're a single-letter edit apart (`Sara`/`Sarah`), or two
edits apart for names over 7 letters. Names of 3 letters or fewer never cluster, since at that
length an edit usually produces a different real name (`Jon`/`Ron`) rather than a typo.

If a flagged pair is genuinely two different characters, add the name to `[continuity].ignore`
in `bluepencil.toml` so it stops clustering:

```toml
[continuity]
ignore = ["Sara"]
```

Add `--json` for machine-readable output.

## Scope

This checks spelling and capitalization consistency only. It does not track physical
descriptions, timelines, geography, objects, character knowledge, or plot setups/payoffs —
those need a human (or an LLM) reading the manuscript for continuity, not a deterministic
lexical scan.
