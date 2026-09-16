use std::collections::HashMap;

use crate::document::Document;
use crate::lexicon::Lexicons;

pub fn frequency(
    docs: &[Document],
    lex: &Lexicons,
    include_stopwords: bool,
    min_length: usize,
) -> Vec<(String, usize)> {
    let mut map: HashMap<&str, usize> = HashMap::new();
    for doc in docs {
        for w in doc.words() {
            if w.lower.chars().count() < min_length || (!include_stopwords && lex.stopwords.contains(&w.lower)) {
                continue;
            }
            *map.entry(w.lower.as_str()).or_default() += 1;
        }
    }
    let mut out: Vec<(String, usize)> = map.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}
