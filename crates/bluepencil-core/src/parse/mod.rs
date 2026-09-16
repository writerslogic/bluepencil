mod fountain;
mod frontmatter;
mod markdown;
mod plaintext;

use crate::document::{Format, Heading};
use crate::span::Span;

pub struct Parsed {
    pub prose: String,
    pub headings: Vec<Heading>,
    /// Offsets where a new paragraph must start even without a blank line (list items).
    pub breaks: Vec<usize>,
    /// Formats with explicit dialogue structure report it; otherwise quotes are scanned.
    pub dialogue: Option<Vec<Span>>,
}

pub fn parse(source: &str, format: Format) -> Parsed {
    match format {
        Format::Plain => plaintext::parse(source),
        Format::Markdown => markdown::parse(source),
        Format::Fountain => fountain::parse(source),
    }
}

/// Byte buffer that masks markup while preserving offsets and newlines.
struct Mask(Vec<u8>);

impl Mask {
    fn new(source: &str) -> Self {
        Self(source.as_bytes().to_vec())
    }

    // Callers pass char-boundary ranges, so the result stays valid UTF-8.
    fn blank(&mut self, start: usize, end: usize) {
        for b in &mut self.0[start..end] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }

    fn finish(self) -> String {
        String::from_utf8(self.0).expect("masking preserves UTF-8")
    }
}

/// Iterates lines as (start offset, content without line ending).
fn lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    source.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line.trim_end_matches(['\n', '\r']))
    })
}
