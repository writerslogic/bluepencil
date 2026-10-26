use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Serialize;

use super::Finding;
use crate::document::Document;
use crate::span::Span;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Starters {
    pub sentence: Vec<(String, usize)>,
    pub paragraph: Vec<(String, usize)>,
    pub findings: Vec<Finding>,
}

fn ranked(map: HashMap<String, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<_> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

/// Opening words of sentences and paragraphs, plus runs of `run`+ consecutive sentences
/// that open with the same word.
pub fn starters(doc: &Document, run: usize) -> Starters {
    let mut sentence = HashMap::new();
    let mut paragraph = HashMap::new();
    let mut findings = Vec::new();

    for p in &doc.paragraphs {
        if let Some(w) = p.sentences.first().and_then(|s| s.words.first()) {
            *paragraph.entry(w.lower.clone()).or_default() += 1;
        }
    }

    let firsts: Vec<(&str, Span)> =
        doc.sentences().filter_map(|s| s.words.first().map(|w| (w.lower.as_str(), s.span))).collect();
    for (w, _) in &firsts {
        *sentence.entry(w.to_string()).or_default() += 1;
    }

    let mut i = 0;
    while i < firsts.len() {
        let next_heading = doc.headings.partition_point(|heading| heading.span.start <= firsts[i].1.start);
        let section_end = doc.headings.get(next_heading).map_or(doc.source.len(), |heading| heading.span.start);
        let j =
            i + firsts[i..].iter().take_while(|(word, span)| *word == firsts[i].0 && span.start < section_end).count();
        if j - i >= run {
            findings.push(Finding {
                rule: "repeated-starter",
                message: format!("{} sentences in a row start with \"{}\"", j - i, firsts[i].0),
                span: Span::new(firsts[i].1.start, firsts[j - 1].1.end),
            });
        }
        i = j;
    }

    Starters { sentence: ranked(sentence), paragraph: ranked(paragraph), findings }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;

    #[test]
    fn opening_runs_stop_at_headings_without_losing_runs_inside_sections() {
        for count in [2, 3, 4] {
            let passage = "She walked home. ".repeat(count);
            let doc = Document::parse("test.md", format!("# One\n{passage}\n\n# Two\n{passage}"), Format::Markdown);
            let result = starters(&doc, 3);
            assert_eq!(result.findings.len(), if count >= 3 { 2 } else { 0 });
            assert_eq!(result.sentence[0].1, count * 2);
        }
    }
}
