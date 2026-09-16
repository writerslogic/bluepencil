use std::collections::HashMap;

use super::Finding;
use crate::document::Document;
use crate::span::Span;

/// Words used exactly once across all documents, as (document index, finding).
pub fn hapax(docs: &[Document]) -> Vec<(usize, Finding)> {
    let mut seen: HashMap<&str, (usize, usize, Span)> = HashMap::new();
    for (d, doc) in docs.iter().enumerate() {
        for w in doc.words() {
            seen.entry(w.lower.as_str()).or_insert((0, d, w.span)).0 += 1;
        }
    }
    let mut out: Vec<(usize, Finding)> = seen
        .into_iter()
        .filter(|(_, (n, _, _))| *n == 1)
        .map(|(word, (_, d, span))| (d, Finding { rule: "hapax", message: word.to_string(), span }))
        .collect();
    out.sort_by_key(|(d, f)| (*d, f.span));
    out
}
