# tags

Dialogue tags, the attribution verb next to a line of dialogue ("she said", "Mara snapped"), separated into three buckets:

- **plain** -- `said`/`asked` and their inflections
- **adverb** -- a plain tag qualified by an adverb ("she said softly")
- **showy** -- a more expressive verb standing in for `said` ("exclaimed", "snapped")

```sh
bluepencil tags
```

No parser: a tag is whichever word from the bundled tag lists sits within a few words of the quote, in the same paragraph, so a verb further away or across a paragraph break isn't found.

## Options

- `--summary` group by verb instead of location

Add `--json` for machine-readable output.
