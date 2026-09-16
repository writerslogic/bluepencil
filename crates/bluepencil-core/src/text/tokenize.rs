use unicode_segmentation::UnicodeSegmentation;

use super::normalize::fold;
use crate::document::Word;
use crate::span::Span;

const HYPHENS: [&str; 3] = ["-", "\u{2010}", "\u{2011}"];

/// Words within `span`, joining hyphenated compounds the way word processors count them.
pub fn words(text: &str, span: Span) -> Vec<Word> {
    let slice = span.slice(text);
    let mut spans: Vec<Span> = Vec::new();
    for (i, w) in slice.unicode_word_indices() {
        if !w.chars().any(char::is_alphabetic) {
            continue;
        }
        let (start, end) = (span.start + i, span.start + i + w.len());
        if let Some(last) = spans.last_mut()
            && HYPHENS.contains(&&text[last.end..start])
        {
            last.end = end;
            continue;
        }
        spans.push(Span::new(start, end));
    }
    spans.into_iter().map(|s| Word { span: s, lower: fold(s.slice(text)) }).collect()
}
