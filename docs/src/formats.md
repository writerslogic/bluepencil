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

Locations for Plain, Markdown, and Fountain always refer to the original file, so line and
column numbers match your editor.
