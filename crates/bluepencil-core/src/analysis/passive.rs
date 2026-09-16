use super::Finding;
use crate::document::Document;
use crate::lexicon::Lexicons;
use crate::span::Span;

const BE_VERBS: &[&str] = &["am", "is", "are", "was", "were", "be", "being", "been"];

/// A past participle by regular `-ed` inflection, filtered for the common false
/// positives that end in `-ed` but are not verbs in this position (`bed`, `red`,
/// `naked`-style adjectives are caught by length, not this list).
fn is_regular_participle(word: &str) -> bool {
    word.len() > 3 && word.ends_with("ed")
}

fn is_participle(word: &str, lex: &Lexicons) -> bool {
    lex.irregular_participles.contains(word) || is_regular_participle(word)
}

/// `be`-verb followed by a past participle, with at most one intervening adverb
/// (`was quickly forgotten`), the standard heuristic passive-voice construction.
/// No POS tagger: a `be`-verb followed by an unrelated `-ed`/irregular-participle
/// word (as an adjective, e.g. "was tired") reads as a false positive here, same
/// tradeoff every heuristic passive detector (write-good, Hemingway) makes.
pub fn find(doc: &Document, lex: &Lexicons) -> Vec<Finding> {
    let mut findings = Vec::new();
    for sentence in doc.sentences() {
        let words = &sentence.words;
        let mut i = 0;
        while i < words.len() {
            if !BE_VERBS.contains(&words[i].lower.as_str()) {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            if j < words.len() && words[j].lower.ends_with("ly") && !is_participle(&words[j].lower, lex) {
                j += 1;
            }
            if j < words.len() && is_participle(&words[j].lower, lex) {
                let mut end = j;
                if end + 1 < words.len() && words[end + 1].lower == "by" {
                    end += 1;
                }
                findings.push(Finding {
                    rule: "passive",
                    message: words[i..=end].iter().map(|w| w.lower.as_str()).collect::<Vec<_>>().join(" "),
                    span: Span::new(words[i].span.start, words[end].span.end),
                });
                i = end + 1;
                continue;
            }
            i += 1;
        }
    }
    findings
}
