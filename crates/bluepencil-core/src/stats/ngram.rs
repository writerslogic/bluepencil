use std::collections::HashMap;

use crate::document::Word;
use crate::span::Span;

/// Every n-gram within a sentence, keyed by its folded text, with the span of each occurrence.
pub fn collect<'a>(sentences: impl Iterator<Item = &'a [Word]>, n: usize) -> HashMap<String, Vec<Span>> {
    let mut map: HashMap<String, Vec<Span>> = HashMap::new();
    for words in sentences {
        for win in words.windows(n) {
            let key = win.iter().map(|w| w.lower.as_str()).collect::<Vec<_>>().join(" ");
            map.entry(key).or_default().push(Span::new(win[0].span.start, win[n - 1].span.end));
        }
    }
    map
}
