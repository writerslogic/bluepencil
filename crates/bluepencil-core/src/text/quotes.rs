use crate::document::Paragraph;
use crate::span::Span;

/// Spans inside double quotation marks. An unclosed quote runs to the end of its paragraph,
/// matching the convention for dialogue that continues into the next paragraph.
pub fn dialogue_spans(text: &str, paragraphs: &[Paragraph]) -> Vec<Span> {
    let mut out = Vec::new();
    for p in paragraphs {
        let mut open: Option<usize> = None;
        for (i, c) in p.span.slice(text).char_indices() {
            let at = p.span.start + i;
            match (c, open) {
                ('“' | '«', _) => open = Some(at + c.len_utf8()),
                ('"', None) => open = Some(at + 1),
                ('”' | '»' | '"', Some(o)) => {
                    if at > o {
                        out.push(Span::new(o, at));
                    }
                    open = None;
                }
                _ => {}
            }
        }
        if let Some(o) = open
            && p.span.end > o
        {
            out.push(Span::new(o, p.span.end));
        }
    }
    out
}
