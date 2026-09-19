use schemars::JsonSchema;
use serde::Serialize;

use super::Finding;
use crate::document::{Document, Sentence};
use crate::span::Span;

const FIRST: &[&str] = &["i", "me", "my", "mine", "myself", "we", "us", "our", "ours", "ourselves"];
const SECOND: &[&str] = &["you", "your", "yours", "yourself", "yourselves"];
const THIRD: &[&str] = &[
    "he",
    "him",
    "his",
    "himself",
    "she",
    "her",
    "hers",
    "herself",
    "it",
    "its",
    "itself",
    "they",
    "them",
    "their",
    "theirs",
    "themselves",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Pov {
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct PovResult {
    pub dominant: Pov,
    pub first_sentences: usize,
    pub second_sentences: usize,
    pub third_sentences: usize,
    pub findings: Vec<Finding>,
}

/// Classifies a sentence's narration (dialogue excluded, since a character addressing
/// someone as "you" doesn't change the narrator's person) by whichever personal-pronoun set
/// has the most hits; unclassified when no pronoun appears or two sets tie.
fn classify(doc: &Document, sentence: &Sentence) -> Option<Pov> {
    let (mut first, mut second, mut third) = (0u32, 0u32, 0u32);
    for w in &sentence.words {
        if doc.in_dialogue(w.span) {
            continue;
        }
        if FIRST.contains(&w.lower.as_str()) {
            first += 1;
        } else if SECOND.contains(&w.lower.as_str()) {
            second += 1;
        } else if THIRD.contains(&w.lower.as_str()) {
            third += 1;
        }
    }
    let max = first.max(second).max(third);
    if max == 0 {
        return None;
    }
    // A tie between two non-zero counts is ambiguous, not evidence for either.
    let winners = [first == max, second == max, third == max].into_iter().filter(|w| *w).count();
    if winners > 1 {
        return None;
    }
    if first == max {
        Some(Pov::First)
    } else if second == max {
        Some(Pov::Second)
    } else {
        Some(Pov::Third)
    }
}

/// Runs of at least `run` consecutive classified sentences that use a point of view other
/// than the one the document predominantly uses.
pub fn pov(doc: &Document, run: usize) -> PovResult {
    let classified: Vec<(Pov, Span)> = doc.sentences().filter_map(|s| classify(doc, s).map(|p| (p, s.span))).collect();

    let first_sentences = classified.iter().filter(|(p, _)| *p == Pov::First).count();
    let second_sentences = classified.iter().filter(|(p, _)| *p == Pov::Second).count();
    let third_sentences = classified.len() - first_sentences - second_sentences;
    let dominant = [(Pov::First, first_sentences), (Pov::Second, second_sentences), (Pov::Third, third_sentences)]
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(p, _)| p)
        .unwrap_or(Pov::Third);

    let mut findings = Vec::new();
    let mut i = 0;
    while i < classified.len() {
        let j = i + classified[i..].iter().take_while(|(p, _)| *p != dominant).count();
        if j - i >= run && classified[i].0 != dominant {
            let other = classified[i].0;
            findings.push(Finding {
                rule: "pov-drift",
                message: format!("{} sentences drift into {other:?} person", j - i),
                span: Span::new(classified[i].1.start, classified[j - 1].1.end),
            });
        }
        i = j.max(i + 1);
    }

    PovResult { dominant, first_sentences, second_sentences, third_sentences, findings }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;

    fn doc(text: &str) -> Document {
        Document::parse("test.md".to_string(), text.to_string(), Format::Markdown)
    }

    #[test]
    fn finds_no_drift_in_consistent_third_person() {
        let d = doc("She walked home. Her keys jingled. She opened the door.");
        let r = pov(&d, 2);
        assert_eq!(r.dominant, Pov::Third);
        assert!(r.findings.is_empty());
    }

    #[test]
    fn flags_a_first_person_run_in_a_third_person_document() {
        let d = doc("She walked home. Her keys jingled in her pocket. \
             I could not believe my eyes. I had never felt so tired. \
             She opened the door. She stepped inside slowly.");
        let r = pov(&d, 2);
        assert_eq!(r.dominant, Pov::Third);
        assert!(r.findings.iter().any(|f| f.rule == "pov-drift"));
    }

    #[test]
    fn ignores_dialogue_when_classifying() {
        let d = doc("She walked home. \"I am tired,\" she said. She opened the door.");
        let r = pov(&d, 2);
        assert_eq!(r.dominant, Pov::Third);
        assert!(r.findings.is_empty());
    }
}
