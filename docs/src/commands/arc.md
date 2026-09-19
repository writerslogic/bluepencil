# arc

The manuscript's pacing shape: a profile per section (chapter, if you use headings for them) of word count, dialogue ratio, sentence rhythm, and style density (adverbs and passive voice per 1,000 words).

```sh
bluepencil arc chapters/*.md
```

Where `report` gives you one number for the whole manuscript, `arc` shows where the rhythm changes -- a slow, adverb-heavy stretch between two brisk, dialogue-driven ones is invisible in an aggregate but obvious chapter by chapter. Compare two points in the manuscript's history with `git diff` on `--json` output, or by running `arc` against an earlier revision with `git show <rev>:<path> | bluepencil arc -`.

Add `--json` for machine-readable output.
