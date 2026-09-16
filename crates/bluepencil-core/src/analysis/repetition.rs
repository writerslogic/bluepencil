use serde::Serialize;

use crate::document::Document;
use crate::lexicon::Lexicons;
use crate::span::Span;
use crate::stats::ngram;

#[derive(Debug, Clone, Serialize)]
pub struct Repeat {
    pub phrase: String,
    pub words: usize,
    pub count: usize,
    pub spans: Vec<Span>,
}

/// Repeated n-grams, skipping all-stopword phrases and shorter phrases that only occur
/// inside a longer repeat.
pub fn repeats(doc: &Document, lex: &Lexicons, min: usize, max: usize, min_count: usize) -> Vec<Repeat> {
    let mut found: Vec<Repeat> = Vec::new();
    for n in (min..=max).rev() {
        let grams = ngram::collect(doc.sentences().map(|s| s.words.as_slice()), n);
        let mut level = Vec::new();
        for (phrase, spans) in grams {
            if spans.len() < min_count || phrase.split(' ').all(|w| lex.stopwords.contains(w)) {
                continue;
            }
            let fresh = spans.iter().filter(|s| !found.iter().any(|r| r.spans.iter().any(|l| l.contains(**s)))).count();
            if fresh < min_count {
                continue;
            }
            level.push(Repeat { phrase, words: n, count: spans.len(), spans });
        }
        found.extend(level);
    }
    found.sort_by(|a, b| b.count.cmp(&a.count).then(b.words.cmp(&a.words)).then_with(|| a.phrase.cmp(&b.phrase)));
    found
}
