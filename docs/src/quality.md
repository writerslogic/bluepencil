# Quality and limits

bluepencil's deterministic rules are meant to surface passages for a writer to review. They do not score writing quality or decide what to change. Repeated words in dialogue, familiar expressions used literally, and deliberate rhythm can all be worth keeping.

## Reproducible rule check

[`benchmarks/quality/cases.json`](https://github.com/writerslogic/bluepencil/blob/main/benchmarks/quality/cases.json) contains 22 hand-labeled, synthetic sentences for five rules. Run `BLUEPENCIL_BIN=target/debug/bluepencil node benchmarks/quality/run.mjs` after building the CLI. On the current code, the set gives 11 true positives, 1 false positive, 10 true negatives, and no false negatives: 91.7% precision and 100% recall **on this small fixture only**. The remaining false positive is a literal “piece of cake” matched by the cliché phrase list. This is a regression fixture, not a representative estimate for novels or screenplays.

The passive detector uses a lightweight verb pattern and a short list of common state adjectives. It can still mistake an adjective for a passive construction or miss a passive without a nearby *be* verb. Cliché matching is phrase based and does not infer whether a phrase is literal. The browser playground shows a capped set of findings; run the CLI on full manuscripts.

A credible estimate of writer-facing precision will need a larger, consented set of real passages, multiple human annotators, a published labeling guide, and per-rule results. No claim about performance on real manuscripts is made from the synthetic fixture.
