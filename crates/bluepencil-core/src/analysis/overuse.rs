use serde::Serialize;

use crate::document::Document;
use crate::lexicon::Lexicons;

#[derive(Debug, Clone, Serialize)]
pub struct Overused {
    pub word: String,
    pub count: usize,
    pub document_per_million: f64,
    pub baseline_per_million: f64,
    pub ratio: f64,
}

/// Words that occur far more often in the document than in general English usage.
/// Skips words absent from the baseline (too rare or not in the source corpus) and
/// words seen fewer than `min_count` times, since a single occurrence always produces
/// a huge but meaningless ratio against a low baseline frequency.
pub fn overused(docs: &[Document], lex: &Lexicons, min_ratio: f64, min_count: usize) -> Vec<Overused> {
    if lex.english_frequency.is_empty() {
        return Vec::new();
    }
    let mut counts = std::collections::HashMap::<&str, usize>::new();
    let mut total = 0usize;
    for doc in docs {
        for w in doc.words() {
            if lex.stopwords.contains(&w.lower) {
                continue;
            }
            *counts.entry(w.lower.as_str()).or_default() += 1;
            total += 1;
        }
    }
    if total == 0 {
        return Vec::new();
    }
    let mut out: Vec<Overused> = counts
        .into_iter()
        .filter(|(_, count)| *count >= min_count)
        .filter_map(|(word, count)| {
            let baseline = *lex.english_frequency.get(word)?;
            let document_per_million = count as f64 / total as f64 * 1_000_000.0;
            let ratio = document_per_million / baseline;
            (ratio >= min_ratio).then_some(Overused {
                word: word.to_string(),
                count,
                document_per_million,
                baseline_per_million: baseline,
                ratio,
            })
        })
        .collect();
    out.sort_by(|a, b| b.ratio.partial_cmp(&a.ratio).unwrap().then_with(|| a.word.cmp(&b.word)));
    out
}
