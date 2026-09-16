pub mod adverbs;
pub mod cliches;
pub mod counts;
pub mod dialogue;
pub mod distribution;
pub mod diversity;
pub mod echoes;
pub mod filter_words;
pub mod frequency;
pub mod hapax;
pub mod hedges;
pub mod outline;
pub mod passive;
pub mod readability;
pub mod repetition;
pub mod rhythm;
pub mod starters;
pub mod tics;

use serde::Serialize;

use crate::document::Document;
use crate::lexicon::PhraseSet;
use crate::span::Span;

/// A located observation a writer may want to act on.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub message: String,
    pub span: Span,
}

pub(crate) fn phrase_findings(doc: &Document, set: &PhraseSet, rule: &'static str) -> Vec<Finding> {
    set.find(doc).into_iter().map(|m| Finding { rule, message: m.phrase, span: m.span }).collect()
}

/// Groups findings by message, most frequent first.
pub fn tally(findings: &[Finding]) -> Vec<(String, usize)> {
    let mut map = std::collections::HashMap::<&str, usize>::new();
    for f in findings {
        *map.entry(f.message.as_str()).or_default() += 1;
    }
    let mut out: Vec<(String, usize)> = map.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}
