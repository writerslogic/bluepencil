use serde::Serialize;

use super::Finding;
use crate::document::Document;
use crate::span::Span;
use crate::stats::Summary;

#[derive(Debug, Clone, Serialize)]
pub struct Rhythm {
    pub lengths: Vec<usize>,
    pub summary: Summary,
    /// Stdev over mean; low values read as monotonous.
    pub variation: f64,
    pub findings: Vec<Finding>,
}

/// Flags runs of `min_run`+ consecutive sentences whose lengths stay within `tolerance` words,
/// and sentences longer than `long`.
pub fn rhythm(doc: &Document, min_run: usize, tolerance: usize, long: usize) -> Rhythm {
    let sentences: Vec<_> = doc.sentences().collect();
    let lengths: Vec<usize> = sentences.iter().map(|s| s.words.len()).collect();
    let summary = Summary::of(&lengths);
    let mut findings = Vec::new();

    let mut i = 0;
    while i < lengths.len() {
        let (mut lo, mut hi) = (lengths[i], lengths[i]);
        let mut j = i + 1;
        while j < lengths.len() {
            let (nlo, nhi) = (lo.min(lengths[j]), hi.max(lengths[j]));
            if nhi - nlo > tolerance {
                break;
            }
            (lo, hi) = (nlo, nhi);
            j += 1;
        }
        if j - i >= min_run {
            findings.push(Finding {
                rule: "monotony",
                message: format!("{} sentences in a row between {lo} and {hi} words", j - i),
                span: Span::new(sentences[i].span.start, sentences[j - 1].span.end),
            });
            i = j;
        } else {
            i += 1;
        }
    }

    for s in &sentences {
        if s.words.len() > long {
            findings.push(Finding { rule: "long-sentence", message: format!("{} words", s.words.len()), span: s.span });
        }
    }
    findings.sort_by_key(|f| f.span);

    let variation = if summary.mean > 0.0 { summary.stdev / summary.mean } else { 0.0 };
    Rhythm { lengths, summary, variation, findings }
}
