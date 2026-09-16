use std::path::Path;

use serde::Serialize;

use crate::location::{LineIndex, Position};
use crate::span::Span;
use crate::{parse, text};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Plain,
    Markdown,
    Fountain,
}

impl Format {
    pub fn from_path(path: &Path) -> Self {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        match ext.as_str() {
            "md" | "markdown" | "mdown" | "mkd" | "mdx" => Self::Markdown,
            "fountain" | "spmd" => Self::Fountain,
            _ => Self::Plain,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Word {
    pub span: Span,
    /// Case-folded form with typographic apostrophes normalized.
    pub lower: String,
}

#[derive(Debug, Clone)]
pub struct Sentence {
    pub span: Span,
    pub words: Vec<Word>,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub span: Span,
    pub sentences: Vec<Sentence>,
    /// Index into `Document::headings` of the section this paragraph belongs to.
    pub section: Option<usize>,
}

impl Paragraph {
    pub fn word_count(&self) -> usize {
        self.sentences.iter().map(|s| s.words.len()).sum()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Heading {
    pub level: u8,
    pub title: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub name: String,
    pub format: Format,
    pub source: String,
    /// Same byte length as `source`, with markup blanked to spaces so offsets line up.
    pub prose: String,
    pub headings: Vec<Heading>,
    pub paragraphs: Vec<Paragraph>,
    pub dialogue: Vec<Span>,
    lines: LineIndex,
}

impl Document {
    pub fn parse(name: impl Into<String>, source: impl Into<String>, format: Format) -> Self {
        let source = source.into();
        let parsed = parse::parse(&source, format);
        let mut paragraphs = text::paragraph::split(&parsed.prose, &parsed.breaks);
        for p in &mut paragraphs {
            p.sentences = text::sentence::split(&parsed.prose, p.span);
            p.section = parsed.headings.iter().rposition(|h| h.span.start < p.span.start);
        }
        paragraphs.retain(|p| !p.sentences.is_empty());
        let dialogue = parsed.dialogue.unwrap_or_else(|| text::quotes::dialogue_spans(&parsed.prose, &paragraphs));
        Self {
            name: name.into(),
            format,
            lines: LineIndex::new(&source),
            source,
            prose: parsed.prose,
            headings: parsed.headings,
            paragraphs,
            dialogue,
        }
    }

    pub fn sentences(&self) -> impl Iterator<Item = &Sentence> {
        self.paragraphs.iter().flat_map(|p| &p.sentences)
    }

    pub fn words(&self) -> impl Iterator<Item = &Word> {
        self.sentences().flat_map(|s| &s.words)
    }

    pub fn word_count(&self) -> usize {
        self.sentences().map(|s| s.words.len()).sum()
    }

    pub fn text(&self, span: Span) -> &str {
        span.slice(&self.source)
    }

    pub fn position(&self, offset: usize) -> Position {
        self.lines.position(&self.source, offset)
    }

    pub fn in_dialogue(&self, span: Span) -> bool {
        let i = self.dialogue.partition_point(|d| d.end < span.end);
        self.dialogue.get(i).is_some_and(|d| d.contains(span))
    }

    pub fn section_title(&self, index: Option<usize>) -> &str {
        index.and_then(|i| self.headings.get(i)).map_or("(untitled)", |h| h.title.as_str())
    }
}
