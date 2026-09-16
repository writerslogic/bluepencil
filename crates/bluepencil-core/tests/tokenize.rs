use bluepencil_core::{Document, Format};

fn words(text: &str) -> Vec<String> {
    Document::parse("t", text, Format::Plain).words().map(|w| w.lower.clone()).collect()
}

#[test]
fn contractions_and_hyphenated_compounds_are_single_words() {
    assert_eq!(words("Don\u{2019}t call a well-known man-of-war."), ["don't", "call", "a", "well-known", "man-of-war"]);
}

#[test]
fn dashes_and_numbers_are_not_words() {
    assert_eq!(words("Wait\u{2014}no - 1987 was the year."), ["wait", "no", "was", "the", "year"]);
}

#[test]
fn quoted_dialogue_is_detected_across_paragraphs() {
    let doc = Document::parse("t", "\u{201c}One two\n\n\u{201c}Three.\u{201d} Four.", Format::Plain);
    let spoken: Vec<&str> = doc.words().filter(|w| doc.in_dialogue(w.span)).map(|w| w.lower.as_str()).collect();
    assert_eq!(spoken, ["one", "two", "three"]);
}
