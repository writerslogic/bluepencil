# Input formats

The format is chosen by file extension. Use `--as plain|markdown|fountain|docx` to override it. Text on stdin is treated as Markdown unless `--as` says otherwise (`--as docx` on stdin is rejected, since docx is a binary archive, not text).

## Markdown (`.md`, `.markdown`, `.mdx`)

Front matter, fenced code, inline code, HTML tags and comments, link targets, images, footnote references, tables, and emphasis markers are ignored. Headings define sections for `outline`, `readability`, and `dialogue`, and are not counted as prose.

## Fountain (`.fountain`, `.spmd`)

Scene headings and sections become sections. Character cues, parentheticals, transitions, notes, and the title page are ignored. Dialogue blocks are measured as dialogue.

## Plain text

Everything is prose. Blank lines separate paragraphs.

## Word (`.docx`)

A `.docx` file is converted to synthetic Markdown before analysis: each paragraph's text is
extracted, and a paragraph styled `Heading1`..`Heading6` or `Title` becomes a `#`..`######`
heading. It is then analyzed exactly like a Markdown document. Tables, images, headers and
footers, comments, and tracked changes are not read. Because the text is reconstructed rather
than read byte-for-byte, locations are approximate and do not map back to positions in the
original file the way they do for the other formats.

## Writing app files

bluepencil also opens these native files directly:

| App | Input | What is read |
| --- | --- | --- |
| Scrivener 3 | `.scriv` project folder | Draft binder items and their RTF text, in binder order. Research and trash are skipped. |
| Final Draft | `.fdx` | Scene headings, action, character cues, parentheticals, and dialogue, reconstructed as Fountain. |
| Ulysses | `.ulyz` sheet or `.ulysses` package folder | Sheet body text. Inline markup text is retained. |
| Tinderbox | `.tbx` | Text of each note, with nested note names preserved in the report path. |
| Pages | `.pages` file or package on macOS | Pages opens the file and exports a temporary Word document for analysis. Pages must be installed. |

These files are read without changing the originals. The Pages importer opens the file in the Pages app and may trigger a macOS permission prompt. Native package locations refer to reconstructed text; they do not map directly to the app's own editor positions. Import size is limited to 64 MiB per source file or extracted sheet. The browser playground handles Markdown, Fountain, and plain text only; use the CLI for writing app files.

The importers have also been checked against public [Scrivener 3](https://github.com/jiayun/scrivener-rs/tree/main/tests/fixtures/sample.scriv), [Final Draft](https://github.com/rsdoiel/fdx/blob/main/testdata/sample-01.fdx), and [Tinderbox](https://github.com/natematias/Collaboration-With-Tinderbox/blob/master/Tinderbox-Collaboration-Template.tbx) samples. A `.pages` document created in Pages was imported locally with its two paragraphs intact and the source file unchanged; Pages is not exercised in CI. Ulysses is covered by constructed package and archive fixtures but has not been checked with a file from the app.

Locations for Plain, Markdown, and Fountain refer to the original file, so line and column numbers match your editor.
