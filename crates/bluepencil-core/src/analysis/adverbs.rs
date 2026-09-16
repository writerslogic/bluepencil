use super::Finding;
use crate::document::Document;
use crate::lexicon::Lexicons;

/// `-ly` adverbs, skipping common adjectives and nouns that share the ending.
pub fn find(doc: &Document, lex: &Lexicons) -> Vec<Finding> {
    doc.words()
        .filter(|w| w.lower.len() > 4 && w.lower.ends_with("ly") && !lex.not_adverbs.contains(&w.lower))
        .map(|w| Finding { rule: "adverb", message: w.lower.clone(), span: w.span })
        .collect()
}
