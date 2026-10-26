use schemars::JsonSchema;
use serde::Serialize;

use super::Finding;
use crate::document::Document;
use crate::span::Span;
use crate::stats::Summary;

#[derive(Debug, Clone, Serialize, JsonSchema)]
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
        let next_heading = doc.headings.partition_point(|heading| heading.span.start <= sentences[i].span.start);
        let section_end = doc.headings.get(next_heading).map_or(doc.source.len(), |heading| heading.span.start);
        let (mut lo, mut hi) = (lengths[i], lengths[i]);
        let mut j = i + 1;
        while j < lengths.len() && sentences[j].span.start < section_end {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;

    #[test]
    fn rhythm_runs_stop_at_headings_without_losing_runs_inside_sections() {
        for count in [3, 4, 5] {
            let passage = "The rain fell. ".repeat(count);
            let doc = Document::parse("test.md", format!("# One\n{passage}\n\n# Two\n{passage}"), Format::Markdown);
            let result = rhythm(&doc, 4, 3, 40);
            assert_eq!(result.findings.len(), if count >= 4 { 2 } else { 0 });
        }
    }
}
