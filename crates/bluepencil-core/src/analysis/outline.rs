use serde::Serialize;

use crate::document::Document;

#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub level: u8,
    pub title: String,
    pub line: usize,
    pub words: usize,
}

pub fn outline(doc: &Document) -> Vec<Section> {
    let mut words = vec![0usize; doc.headings.len()];
    let mut preamble = 0;
    for p in &doc.paragraphs {
        match p.section {
            Some(i) => words[i] += p.word_count(),
            None => preamble += p.word_count(),
        }
    }
    let mut out = Vec::new();
    if preamble > 0 || doc.headings.is_empty() {
        out.push(Section { level: 0, title: "(before first heading)".into(), line: 1, words: preamble });
    }
    out.extend(doc.headings.iter().zip(words).map(|(h, w)| Section {
        level: h.level,
        title: h.title.clone(),
        line: doc.position(h.span.start).line,
        words: w,
    }));
    out
}
