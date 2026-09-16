# Word lists

bluepencil ships with lists of function words, filter words, hedges, cliches, and common -ly words that are not adverbs. You can extend or trim them in `bluepencil.toml`:

```toml
[lexicon]
filter = ["glanced"]
cliches = ["in the blink of an eye"]
ignore = ["just"]

[lexicon.files]
tics = ["lists/tics.txt"]
```

List files have one word or phrase per line. Lines starting with `#` are comments. Matching is case-insensitive and treats curly and straight apostrophes the same.
