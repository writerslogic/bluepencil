# Input formats

The format is chosen by file extension. Use `--as plain|markdown|fountain` to override it. Text on stdin is treated as Markdown unless `--as` says otherwise.

## Markdown (`.md`, `.markdown`, `.mdx`)

Front matter, fenced code, inline code, HTML tags and comments, link targets, images, footnote references, tables, and emphasis markers are ignored. Headings define sections for `outline`, `readability`, and `dialogue`, and are not counted as prose.

## Fountain (`.fountain`, `.spmd`)

Scene headings and sections become sections. Character cues, parentheticals, transitions, notes, and the title page are ignored. Dialogue blocks are measured as dialogue.

## Plain text

Everything is prose. Blank lines separate paragraphs.

Locations always refer to the original file, so line and column numbers match your editor.
