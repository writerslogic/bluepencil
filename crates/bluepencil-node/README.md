# bluepencil (Node.js)

Native Node.js bindings for [bluepencil](https://github.com/writerslogic/bluepencil), the prose analysis toolkit for writers. Same engine as the CLI, no subprocess, no config file: pass text in, get located findings back.

```sh
npm install bluepencil
```

Prebuilt binaries ship for macOS (x64, arm64), Linux (x64, arm64), and Windows (x64). Node 18 or newer.

```js
import { checkStyle, findEchoes, report } from 'bluepencil';

const style = checkStyle(text, { format: 'plain' });
// style.counts.adverbs, style.perThousand.passive, style.findings[0].location.line ...

const echoes = findEchoes(text, { window: 50, minLength: 4 });
const summary = report(text);
```

| Function | Returns |
|---|---|
| `checkStyle(text, options?)` | Adverbs, filter words, hedges, clichés, tics, passive voice, nominalizations: counts, rates per 1,000 words, and every finding with `line`, `column`, and an excerpt |
| `findEchoes(text, options?)` | The same word reappearing within a window of words |
| `findRepeats(text, options?)` | Multi-word phrases that recur, with every location |
| `analyzeRhythm(text, options?)` | Sentence-length distribution, variation, monotonous runs, overlong sentences |
| `sentenceStarters(text, options?)` | Most common sentence and paragraph openers, and runs that open the same way |
| `analyzeDialogue(text, options?)` | Dialogue versus narration overall and per section; every quoted line with its tag classified as plain, showy, or adverb-modified |
| `sectionArc(text, options?)` | Per-heading pacing profile: words, dialogue share, sentence length, adverb and passive density |
| `report(text, options?)` | Everything at a glance, matching `bluepencil report` |

Every function takes `format: "markdown"` (default), `"plain"`, or `"fountain"`. Defaults for windows, runs, and thresholds match `bluepencil init`; see `index.d.ts` for each option.

## Building from source

```sh
cd crates/bluepencil-node
npm install
npm run build        # release addon for the host platform
npm test
```

Licensed under MIT OR Apache-2.0, like the rest of the workspace.
