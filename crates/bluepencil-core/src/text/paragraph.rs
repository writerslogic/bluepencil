use crate::document::Paragraph;
use crate::span::Span;

/// Splits on blank lines and on forced `breaks` (such as list items).
pub fn split(text: &str, breaks: &[usize]) -> Vec<Paragraph> {
    let mut out = Vec::new();
    let mut current: Option<Span> = None;
    let mut offset = 0;
    let flush = |cur: &mut Option<Span>, out: &mut Vec<Paragraph>| {
        if let Some(s) = cur.take() {
            out.push(Paragraph { span: s, sentences: Vec::new(), section: None });
        }
    };

    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(&mut current, &mut out);
            continue;
        }
        let lead = start + (line.len() - line.trim_start().len());
        let tail = lead + trimmed.len();
        if breaks.iter().any(|&b| b >= start && b <= lead) {
            flush(&mut current, &mut out);
        }
        match current.as_mut() {
            Some(s) => s.end = tail,
            None => current = Some(Span::new(lead, tail)),
        }
    }
    flush(&mut current, &mut out);
    out
}
