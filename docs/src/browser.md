# The browser writing desk

[Open the writing desk](../) to edit and analyze English prose locally. Paste a manuscript or open a UTF-8 Markdown, Fountain, or plain text file up to 16 MiB. The engine parses the whole document and scans groups of rules in a background worker, with progress and cancellation. Chapter boundaries, Markdown fences, and dialogue retain their context.

## Observations in the text

Colored backgrounds and underlines mark observations near the visible passage, with a buffer above and below. Scrolling updates the highlights and toolbar counts. The editor is virtualized, so large manuscripts do not require every line and finding to be rendered at once.

| Toolbar group | Observations |
| --- | --- |
| Repetition | Nearby word echoes, repeated phrases, repeated sentence openings |
| Style | Wordy constructions, familiar phrases, possible passive constructions, perception and thought words, hedges, adverbs |
| Rhythm | Long sentences and runs of similar sentence lengths |
| Dialogue | Expressive and adverb-modified dialogue tags |
| Narrative | Possible tense and grammatical-person shifts |

Use the toolbar groups to turn individual types on or off. Click a marked passage to open its explanation above the editor; overlapping observations can be inspected separately. The explanation stays on the selected observation while you scroll. **Return to this passage** takes you back, and the arrows move among nearby observations. **Inspect finding** opens a nearby observation. `Alt+Enter` inspects an observation at the cursor; `Escape` closes the explanation.

At most 1,000 enabled observations are drawn in one buffered view, with a notice when that display limit is reached. The complete analysis stays in the worker, so scrolling retrieves findings throughout the manuscript. Turning off types can reduce a dense view. The VS Code adapter retains its separate 1 MiB input and 150-observation display limits.

## Manuscript chart

**Analysis chart** opens a panel below the editor. Its toolbar switches each line independently.

| Line | Value at a node |
| --- | --- |
| Sentence length | Mean words per sentence |
| Paragraph length | Mean full paragraph length for distinct paragraphs touching the interval |
| Dialogue share | Percentage of words inside dialogue |
| Finding density | All observations starting in the interval, per 1,000 words |
| Sentence variation | Standard deviation of sentence lengths divided by their mean, expressed as a percentage |

Nodes group consecutive sentences into approximate word intervals. **Auto interval** adapts to the text; explicit intervals are available, with a larger interval used when needed to keep the chart below 250 nodes. A very long sentence can occupy a node by itself. Values include the full sentences touching the selected passage.

**Entire manuscript** gives an overview, with the visible passage and buffer shaded. **Visible passage + buffer** follows the editor as you scroll. Each line is scaled independently within the displayed range. Hover over a node or use the point slider to read actual values and units. Click the chart or select **Go to passage** to move the editor there. These are measurable pacing signals, not a score for narrative pace. Finding density includes all rule types, independently of which highlights are enabled.

Panels resize the editor instead of covering it. On small screens, opening one panel closes the other to preserve room for reading.

## Editing and limits

Undo works for edits and sample replacements. **Download** saves the current text; the browser does not write changes back to an opened file. Text stays in the tab's memory, and the page requests confirmation before leaving an edited document.

Conversational perception words and sensory descriptions such as “smelled of rain” are suppressed. Dialogue does not create narration-rhythm or repeated-opening findings, and runs stop at headings. Repeated phrases need three occurrences. Narrative observations need three classified sentences differing from a baseline supported by at least 80% of twelve or more classified narration sentences. These remain heuristics: deliberate repetition, changes of tense or person, and character voice may be worth keeping. See [Quality and limits](quality.md).

For native writing-app formats, project configuration, and scriptable reports, use the [CLI](installation.md).
