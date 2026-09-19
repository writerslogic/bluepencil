use schemars::JsonSchema;
use serde::Serialize;

use super::{adverbs, passive};
use crate::document::Document;
use crate::lexicon::Lexicons;
use crate::span::Span;
use crate::stats::per_thousand;

/// A section's pacing profile: the density signals a reader would feel as rhythm, not just
/// a length. Two chapters can share a word count and read completely differently.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SectionArc {
    pub section: String,
    pub words: usize,
    pub dialogue_ratio: f64,
    pub sentence_words_mean: f64,
    pub adverbs: usize,
    pub adverbs_per_1k: f64,
    pub passive: usize,
    pub passive_per_1k: f64,
}

#[derive(Default)]
struct Bucket {
    words: usize,
    dialogue_words: usize,
    sentence_words: Vec<usize>,
    adverbs: usize,
    passive: usize,
}

/// The manuscript's pacing shape, one row per section (chapter, if you use headings for
/// them). Unlike a whole-document `report`, this shows where the rhythm changes: a slow,
/// adverb-heavy stretch between two brisk, dialogue-driven ones is invisible in an
/// aggregate but obvious chapter by chapter.
pub fn arc(doc: &Document, lex: &Lexicons) -> Vec<SectionArc> {
    let mut buckets: Vec<(Option<usize>, Bucket)> = Vec::new();
    for p in &doc.paragraphs {
        let bucket = match buckets.last_mut() {
            Some((sec, b)) if *sec == p.section => b,
            _ => {
                buckets.push((p.section, Bucket::default()));
                &mut buckets.last_mut().unwrap().1
            }
        };
        for s in &p.sentences {
            bucket.sentence_words.push(s.words.len());
            bucket.words += s.words.len();
            bucket.dialogue_words += s.words.iter().filter(|w| doc.in_dialogue(w.span)).count();
        }
    }

    let section_of = |span: Span| doc.paragraphs.iter().find(|p| p.span.contains(span)).and_then(|p| p.section);
    for f in adverbs::find(doc, lex) {
        let sec = section_of(f.span);
        if let Some((_, b)) = buckets.iter_mut().find(|(s, _)| *s == sec) {
            b.adverbs += 1;
        }
    }
    for f in passive::find(doc, lex) {
        let sec = section_of(f.span);
        if let Some((_, b)) = buckets.iter_mut().find(|(s, _)| *s == sec) {
            b.passive += 1;
        }
    }

    buckets
        .into_iter()
        .map(|(sec, b)| {
            let mean = if b.sentence_words.is_empty() {
                0.0
            } else {
                b.sentence_words.iter().sum::<usize>() as f64 / b.sentence_words.len() as f64
            };
            let dialogue_ratio = if b.words == 0 { 0.0 } else { b.dialogue_words as f64 / b.words as f64 };
            SectionArc {
                section: doc.section_title(sec).to_string(),
                words: b.words,
                dialogue_ratio,
                sentence_words_mean: mean,
                adverbs: b.adverbs,
                adverbs_per_1k: per_thousand(b.adverbs, b.words),
                passive: b.passive,
                passive_per_1k: per_thousand(b.passive, b.words),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;

    fn doc(text: &str) -> Document {
        Document::parse("test.md".to_string(), text.to_string(), Format::Markdown)
    }

    #[test]
    fn splits_profile_by_section() {
        let lex = Lexicons::default();
        let d = doc(
            "# One\n\n\"Go now,\" she said. \"Please.\"\n\n# Two\n\nShe walked slowly and quietly through the long, empty hall.",
        );
        let rows = arc(&d, &lex);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].section, "One");
        assert!(rows[0].dialogue_ratio > 0.5, "chapter one should read as dialogue-heavy: {rows:?}");
        assert_eq!(rows[1].section, "Two");
        assert!(rows[1].adverbs >= 2, "chapter two should flag its adverbs: {rows:?}");
    }

    #[test]
    fn a_document_with_no_headings_is_one_section() {
        let lex = Lexicons::default();
        let d = doc("She walked home. She opened the door.");
        let rows = arc(&d, &lex);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].section, "(untitled)");
        assert_eq!(rows[0].words, 7);
    }
}
