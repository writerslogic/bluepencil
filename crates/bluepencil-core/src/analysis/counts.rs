use serde::Serialize;

use crate::document::Document;

pub const READING_WPM: f64 = 238.0;
pub const SPEAKING_WPM: f64 = 150.0;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Counts {
    pub words: usize,
    pub characters: usize,
    pub characters_no_spaces: usize,
    pub sentences: usize,
    pub paragraphs: usize,
    pub reading_minutes: f64,
    pub speaking_minutes: f64,
}

impl Counts {
    pub fn of(doc: &Document) -> Self {
        let (mut chars, mut nospace) = (0, 0);
        for p in &doc.paragraphs {
            for c in p.span.slice(&doc.prose).chars() {
                chars += 1;
                if !c.is_whitespace() {
                    nospace += 1;
                }
            }
        }
        let words = doc.word_count();
        Self {
            words,
            characters: chars,
            characters_no_spaces: nospace,
            sentences: doc.sentences().count(),
            paragraphs: doc.paragraphs.len(),
            reading_minutes: words as f64 / READING_WPM,
            speaking_minutes: words as f64 / SPEAKING_WPM,
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.words += other.words;
        self.characters += other.characters;
        self.characters_no_spaces += other.characters_no_spaces;
        self.sentences += other.sentences;
        self.paragraphs += other.paragraphs;
        self.reading_minutes += other.reading_minutes;
        self.speaking_minutes += other.speaking_minutes;
        self
    }
}
