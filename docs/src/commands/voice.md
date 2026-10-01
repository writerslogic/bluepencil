# voice

Does each character sound like themselves? The model reads all the dialogue, writes a short profile of how each speaking character talks as the page actually shows it, and flags lines that break the pattern: a line that sounds like a different character, like the narrator, or like no one in particular.

```sh
bluepencil voice chapters/*.md
```

```text
Voices
  Tom (212 lines)
      Short sentences, heavy contractions, never a word over three syllables. Deflects with jokes.
  Mara (140 lines)
      Full sentences, formal register, no slang. Asks questions instead of stating.

Off-voice lines
  ch09.md:44:3  Tom
      "I find that an indubitably compelling proposition."
      → Latinate and formal; nothing else Tom says sounds like this. It reads as Mara's line.

1 off-voice lines across 2 speaking characters
```

Each flagged line is a quote the model cited, pinned to its real `file:line:column` in your files. A quote that cannot be found verbatim is marked and shown at the cited line instead.

Add `--json` for machine-readable output.

This command sends the manuscript to the Claude API and needs an API key; see `[model]` in [Configuration](../configuration.md). Where attribution can be inferred mechanically, the local [speakers](speakers.md) command counts who speaks and how much without a model.
