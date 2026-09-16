#![no_main]

use bluepencil_core::{Document, Format};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else { return };
    let Ok(source) = std::str::from_utf8(rest) else { return };
    let format = match selector % 3 {
        0 => Format::Plain,
        1 => Format::Markdown,
        _ => Format::Fountain,
    };
    let doc = Document::parse("fuzz", source, format);
    // Exercised for the panics/OOB indexing they could hide, not their output:
    // every span a parser hands back must slice cleanly out of `prose`.
    for word in doc.words() {
        let _ = word.span.slice(&doc.prose);
    }
    for heading in &doc.headings {
        let _ = heading.span.slice(&doc.prose);
    }
});
