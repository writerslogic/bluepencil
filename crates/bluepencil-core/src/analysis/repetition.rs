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
    // Spans already claimed by a longer repeat (n descends from `max`), sorted by start.
    // Bounded by `max_len`, the widest span seen, so a containment query only has to
    // scan the handful of covered spans that could possibly reach far enough to contain
    // it, instead of every span found so far.
    let mut covered: Vec<Span> = Vec::new();
    let mut max_len: usize = 0;
    for n in (min..=max).rev() {
        let grams = ngram::collect(doc.sentences().map(|s| s.words.as_slice()), n);
        let mut level = Vec::new();
        for (phrase, spans) in grams {
            if spans.len() < min_count || phrase.split(' ').all(|w| lex.stopwords.contains(w)) {
                continue;
            }
            let fresh = spans.iter().filter(|s| !is_covered(&covered, max_len, **s)).count();
            if fresh < min_count {
                continue;
            }
            for &s in &spans {
                max_len = max_len.max(s.len());
            }
            level.push(Repeat { phrase, words: n, count: spans.len(), spans });
        }
        covered.extend(level.iter().flat_map(|r| r.spans.iter().copied()));
        covered.sort_unstable_by_key(|s| s.start);
        found.extend(level);
    }
    found.sort_by(|a, b| b.count.cmp(&a.count).then(b.words.cmp(&a.words)).then_with(|| a.phrase.cmp(&b.phrase)));
    found
}

/// Whether any span in `covered` (sorted by `start`, none longer than `max_len`) contains `s`.
/// A containing span must start no earlier than `s.end - max_len` (or it couldn't reach
/// `s.end`) and no later than `s.start`, so the search window is bounded regardless of
/// how many spans have been found so far.
fn is_covered(covered: &[Span], max_len: usize, s: Span) -> bool {
    let lo = covered.partition_point(|c| c.start < s.end.saturating_sub(max_len));
    let hi = covered.partition_point(|c| c.start <= s.start);
    covered[lo..hi].iter().any(|c| c.contains(s))
}
