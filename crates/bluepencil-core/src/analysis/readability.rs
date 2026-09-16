use serde::Serialize;

use crate::document::{Document, Sentence};
use crate::stats::readability::{Readability, score};

#[derive(Debug, Clone, Serialize)]
pub struct SectionReadability {
    pub title: String,
    pub scores: Readability,
}

pub fn document(doc: &Document) -> Readability {
    score(doc.sentences(), &doc.source)
}

pub fn sections(doc: &Document) -> Vec<SectionReadability> {
    let mut groups: Vec<(Option<usize>, Vec<&Sentence>)> = Vec::new();
    for p in &doc.paragraphs {
        match groups.last_mut() {
            Some((sec, list)) if *sec == p.section => list.extend(&p.sentences),
            _ => groups.push((p.section, p.sentences.iter().collect())),
        }
    }
    groups
        .into_iter()
        .map(|(sec, list)| SectionReadability {
            title: doc.section_title(sec).to_string(),
            scores: score(list.into_iter(), &doc.source),
        })
        .collect()
}
