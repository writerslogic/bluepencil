use schemars::JsonSchema;
use serde::Serialize;

use super::Finding;
use crate::document::{Document, Sentence};
use crate::span::Span;

const PAST_AUX: &[&str] = &["was", "were", "had", "did"];
const PRESENT_AUX: &[&str] = &["is", "are", "am", "does", "has", "have"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Tense {
    Past,
    Present,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TenseResult {
    pub dominant: Tense,
    pub past_sentences: usize,
    pub present_sentences: usize,
    pub findings: Vec<Finding>,
}

/// Classifies a sentence's narration (dialogue excluded, since a character can speak in any
/// tense without the narration drifting) as `Past`, `Present`, or unclassified when neither
/// signal is present or the two tie. Past is signaled by a past-tense auxiliary or a regular
/// `-ed` inflection; present by a present-tense auxiliary. There's no POS tagger, so a
/// present-tense main verb without an auxiliary ("she walks") is invisible to this heuristic,
/// the same asymmetry every auxiliary-based tense heuristic has.
fn classify(doc: &Document, sentence: &Sentence) -> Option<Tense> {
    let mut past = 0u32;
    let mut present = 0u32;
    for w in &sentence.words {
        if doc.in_dialogue(w.span) {
            continue;
        }
        if PAST_AUX.contains(&w.lower.as_str()) || (w.lower.len() > 3 && w.lower.ends_with("ed")) {
            past += 1;
        } else if PRESENT_AUX.contains(&w.lower.as_str()) {
            present += 1;
        }
    }
    match past.cmp(&present) {
        std::cmp::Ordering::Greater => Some(Tense::Past),
        std::cmp::Ordering::Less => Some(Tense::Present),
        std::cmp::Ordering::Equal => None,
    }
}

/// Runs of at least `run` consecutive classified sentences that use the tense the document
/// doesn't predominantly use.
pub fn tense(doc: &Document, run: usize) -> TenseResult {
    let classified: Vec<(Tense, Span)> =
        doc.sentences().filter_map(|s| classify(doc, s).map(|t| (t, s.span))).collect();

    let past_sentences = classified.iter().filter(|(t, _)| *t == Tense::Past).count();
    let present_sentences = classified.len() - past_sentences;
    let dominant = if present_sentences > past_sentences { Tense::Present } else { Tense::Past };

    let mut findings = Vec::new();
    let mut i = 0;
    while i < classified.len() {
        let j = i + classified[i..].iter().take_while(|(t, _)| *t != dominant).count();
        if j - i >= run && classified[i].0 != dominant {
            let other = classified[i].0;
            findings.push(Finding {
                rule: "tense-drift",
                message: format!("{} sentences drift into {other:?} tense", j - i),
                span: Span::new(classified[i].1.start, classified[j - 1].1.end),
            });
        }
        i = j.max(i + 1);
    }

    TenseResult { dominant, past_sentences, present_sentences, findings }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;

    fn doc(text: &str) -> Document {
        Document::parse("test.md".to_string(), text.to_string(), Format::Markdown)
    }

    #[test]
    fn finds_no_drift_in_consistent_past_tense() {
        let d = doc("She walked home. She opened the door. She was tired.");
        let r = tense(&d, 2);
        assert_eq!(r.dominant, Tense::Past);
        assert!(r.findings.is_empty());
    }

    #[test]
    fn flags_a_present_tense_run_in_a_past_tense_document() {
        let d = doc("She walked home yesterday. She opened the door then. \
             She is happy now. She is glad now. \
             She had finished the letter. She had mailed it.");
        let r = tense(&d, 2);
        assert_eq!(r.dominant, Tense::Past);
        assert!(r.findings.iter().any(|f| f.rule == "tense-drift"));
    }

    #[test]
    fn ignores_dialogue_when_classifying() {
        let d = doc("She walked home. \"I am happy,\" she said. She opened the door.");
        let r = tense(&d, 2);
        assert_eq!(r.dominant, Tense::Past);
        assert!(r.findings.is_empty());
    }
}
