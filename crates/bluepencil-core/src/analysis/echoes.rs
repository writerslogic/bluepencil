use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::document::Document;
use crate::lexicon::{Lexicons, WordSet};
use crate::span::Span;
use crate::text::lemma::stem;

#[derive(Debug, Clone, Serialize)]
pub struct Echo {
    pub word: String,
    pub first: Span,
    pub second: Span,
    pub distance: usize,
}

/// Words that are always capitalized except at the start of a sentence (names, places).
pub fn proper_nouns(doc: &Document) -> HashSet<&str> {
    let mut capitalized = HashSet::new();
    let mut lowercase = HashSet::new();
    for s in doc.sentences() {
        for w in s.words.iter().skip(1) {
            if doc.text(w.span).starts_with(char::is_uppercase) {
                capitalized.insert(w.lower.as_str());
            } else {
                lowercase.insert(w.lower.as_str());
            }
        }
    }
    capitalized.retain(|w| !lowercase.contains(w));
    capitalized
}

/// A content word reappearing within `window` words of its previous use.
pub fn echoes(
    doc: &Document,
    lex: &Lexicons,
    window: usize,
    min_length: usize,
    ignore: &WordSet,
    include_names: bool,
) -> Vec<Echo> {
    let names = if include_names { HashSet::new() } else { proper_nouns(doc) };
    let mut last: HashMap<String, (usize, Span)> = HashMap::new();
    let mut out = Vec::new();
    for (i, w) in doc.words().enumerate() {
        if w.lower.chars().count() < min_length
            || lex.stopwords.contains(&w.lower)
            || ignore.contains(&w.lower)
            || names.contains(w.lower.as_str())
        {
            continue;
        }
        let key = stem(&w.lower);
        if let Some(&(j, span)) = last.get(&key)
            && i - j <= window
        {
            out.push(Echo { word: key.clone(), first: span, second: w.span, distance: i - j });
        }
        last.insert(key, (i, w.span));
    }
    out
}
