# Quality and limits

bluepencil's deterministic rules are meant to surface passages for a writer to review. They do not score writing quality or decide what to change. Repeated words in dialogue, familiar expressions used literally, and deliberate rhythm can all be worth keeping.

## Reproducible rule check

[`benchmarks/quality/cases.json`](https://github.com/writerslogic/bluepencil/blob/main/benchmarks/quality/cases.json) contains 22 hand-labeled, synthetic sentences for five rules. Run `BLUEPENCIL_BIN=target/debug/bluepencil node benchmarks/quality/run.mjs` after building the CLI. On the current code, the set gives 11 true positives, 1 false positive, 10 true negatives, and no false negatives: 91.7% precision and 100% recall **on this small fixture only**. The remaining false positive is a literal “piece of cake” matched by the cliché phrase list. This is a regression fixture, not a representative estimate for novels or screenplays.

The passive detector uses a lightweight verb pattern and a short list of common state adjectives. It can still mistake an adjective for a passive construction or miss a passive without a nearby *be* verb. Cliché matching is phrase based and does not infer whether a phrase is literal.

## Browser observations

The [browser writing desk](browser.md) analyzes documents up to 16 MiB and retrieves findings near the visible text. Its display limit applies to a buffered view, rather than the manuscript's beginning. Its contextual filters suppress conversational perception words, sensory descriptions such as “smelled of rain,” and narration patterns caused by dialogue. Tense and person observations require a strong manuscript-wide baseline. These choices have regression coverage in `crates/bluepencil-web/src/manuscript/tests.rs`; they do not establish precision on real manuscripts.

Browser integration checks cover synthetic manuscripts above the former size limit and at the 16 MiB boundary, observations near their endings, scrolling, type switches, charts and navigation, cancellation, and mobile errors. Run the [browser checks](https://github.com/writerslogic/bluepencil/blob/main/CONTRIBUTING.md#browser-writing-desk) to reproduce them. The chart describes sentence length, paragraph length, dialogue, and observation density; it does not measure narrative quality or infer scene pace.

A credible estimate of writer-facing precision will need a larger, consented set of real passages, multiple human annotators, a published labeling guide, and per-rule results. No claim about performance on real manuscripts is made from the synthetic fixture.
