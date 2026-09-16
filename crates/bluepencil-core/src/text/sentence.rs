use super::abbreviations::is_abbreviation;
use super::tokenize;
use crate::document::Sentence;
use crate::span::Span;

fn is_terminal(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '…' | '‽')
}

fn is_closer(c: char) -> bool {
    matches!(c, '"' | '\'' | '”' | '’' | ')' | ']' | '»' | '*' | '_')
}

fn is_opener(c: char) -> bool {
    matches!(c, '"' | '\'' | '“' | '‘' | '(' | '[' | '«' | '¿' | '¡')
}

/// Splits a paragraph into sentences.
///
/// A boundary needs terminal punctuation, optional closing quotes, whitespace, and then a
/// capital, digit, or opening quote. Lowercase continuations (`"Why?" she asked.`) and
/// abbreviations (`Mr.`, `U.S.`, initials) do not end a sentence.
pub fn split(text: &str, para: Span) -> Vec<Sentence> {
    let s = para.slice(text);
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let mut bounds = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < chars.len() {
        let (pos, c) = chars[i];
        if !is_terminal(c) {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < chars.len() && is_terminal(chars[j].1) {
            j += 1;
        }
        while j < chars.len() && is_closer(chars[j].1) {
            j += 1;
        }
        if j >= chars.len() {
            break;
        }
        if !chars[j].1.is_whitespace() {
            i = j;
            continue;
        }
        let end = chars[j].0;
        let mut k = j;
        while k < chars.len() && chars[k].1.is_whitespace() {
            k += 1;
        }
        if k >= chars.len() {
            break;
        }
        let next = chars[k].1;
        let capital = next.is_uppercase() || next.is_ascii_digit() || is_opener(next);
        let abbreviated = c == '.' && j == i + 1 && is_abbreviation(token_before(s, pos));
        if capital && !abbreviated {
            bounds.push((start, end));
            start = chars[k].0;
        }
        i = k;
    }
    bounds.push((start, s.len()));

    bounds
        .into_iter()
        .filter_map(|(a, b)| {
            let piece = &s[a..b];
            let lead = piece.len() - piece.trim_start().len();
            let span = Span::new(para.start + a + lead, para.start + a + piece.trim_end().len());
            let words = tokenize::words(text, span);
            (!words.is_empty()).then_some(Sentence { span, words })
        })
        .collect()
}

fn token_before(s: &str, pos: usize) -> &str {
    let head = &s[..pos];
    let from = head.rfind(char::is_whitespace).map_or(0, |i| i + head[i..].chars().next().map_or(1, char::len_utf8));
    &head[from..]
}
