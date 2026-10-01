# scenes

A scene audit. The model splits the manuscript into scenes and, for each one, names what the viewpoint character wants, what stands in the way, and what is different at the end. Then it gives a verdict: `keep`, `tighten`, `cut`, or `merge`.

```sh
bluepencil scenes chapters/*.md
```

Before the text, the model is given the pacing profile that [arc](arc.md) measures locally for each section: word count, dialogue share, and mean sentence length. It is evidence, not a verdict, so a long scene with no dialogue is only flagged when the reading agrees.

```text
scene                 starts        verdict
Platform              ch01.md:1:1   tighten
The letter            ch01.md:88:1  keep
Breakfast, again      ch02.md:1:1   merge

ch01.md:1:1  Platform
    goal      Get on the train before her father arrives.
    conflict  The train is late and he is already on the platform.
    change    She boards knowing he saw her.
    → tighten: The confrontation is done by line 60; the last four paragraphs re-describe the platform.

3 scenes, 2 to look at
```

Each scene's start is a quote of its opening words, pinned to its real `file:line:column`.

Add `--json` for machine-readable output.

This command sends the manuscript to the Claude API and needs an API key; see `[model]` in [Configuration](../configuration.md).
