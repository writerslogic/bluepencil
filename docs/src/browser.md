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

**Analysis chart** opens a panel below the editor. Its toolbar switches each metric independently. **Tracks** gives each metric its own aligned row, with actual values and a labeled scale, so lines remain visible. **Overlay** compares their shapes on independent relative scales. All tracks share a position in the text and a cursor.

| Line | Value at a node |
| --- | --- |
| Sentence length | Mean words per sentence |
| Paragraph length | Mean full paragraph length for distinct paragraphs touching the sample |
| Dialogue share | Percentage of words inside dialogue |
| Finding density | All observations starting in the sample, per 1,000 words |
| Sentence variation | Variation over the current and up to four preceding sentences, averaged across the sample; standard deviation divided by mean length, as a percentage |

**Entire manuscript** defaults to eight samples within each detected chapter, with chapter boundaries labeled below the chart. Choose four, eight, or sixteen samples per chapter, or a 128-, 256-, or 512-sample overview. Samples group consecutive sentences; short chapters have fewer nodes. Conventional standalone chapter labels and Markdown or Fountain headings establish boundaries. Without chapter structure, automatic detail uses a 256-sample overview.

**Visible passage + buffer** follows the editor as you scroll, with a node for every sentence by default. Switch to **Every paragraph** to see full paragraph samples. Values include the full sentences or paragraphs touching the selected passage. Detail choices are remembered separately for the two views. Explicit choices are honored; a view exceeding 8,192 samples asks for a coarser setting or a smaller passage.

Shading marks the visible passage and its surrounding buffer. Hover over a node or use the point slider to read its values, sentence and paragraph positions, and word range. Click the chart or select **Go to passage** to move the editor there. Closely spaced markers may be thinned on screen, while every sample remains in the line and accessible through the slider. These are measurable pacing signals, not a score for narrative pace. Finding density includes all rule types, independently of which highlights are enabled. Sentence variation resets at chapter boundaries.

Panels resize the editor instead of covering it. On small screens, opening one panel closes the other to preserve room for reading. The reading area expands on wide displays, up to an 1,800-pixel content column.

## Editing and limits

Undo works for edits and sample replacements. **Download** saves the current text; the browser does not write changes back to an opened file. Text stays in the tab's memory, and the page requests confirmation before leaving an edited document.

Conversational perception words and sensory descriptions such as “smelled of rain” are suppressed. Dialogue does not create narration-rhythm or repeated-opening findings, and runs stop at headings. Repeated phrases need three occurrences. Narrative observations need three classified sentences differing from a baseline supported by at least 80% of twelve or more classified narration sentences. These remain heuristics: deliberate repetition, changes of tense or person, and character voice may be worth keeping. See [Quality and limits](quality.md).

For native writing-app formats, project configuration, and scriptable reports, use the [CLI](installation.md).
