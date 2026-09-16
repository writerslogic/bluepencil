use bluepencil_core::analysis::passive;
use bluepencil_core::{Document, Format, Lexicons};

fn find(text: &str) -> Vec<String> {
    let lex = Lexicons::default();
    let doc = Document::parse("t", text, Format::Plain);
    passive::find(&doc, &lex).into_iter().map(|f| f.message).collect()
}

#[test]
fn regular_participle_with_by_clause() {
    assert_eq!(find("The ball was thrown by John."), ["was thrown by"]);
}

#[test]
fn adverb_between_be_verb_and_participle() {
    assert_eq!(find("She was quickly forgotten."), ["was quickly forgotten"]);
}

#[test]
fn irregular_participle_after_been() {
    assert_eq!(find("The window had been broken."), ["been broken"]);
}

#[test]
fn active_voice_has_no_findings() {
    assert!(find("John threw the ball. She forgot quickly.").is_empty());
}

#[test]
fn be_verb_alone_is_not_flagged() {
    assert!(find("I am happy. They were here.").is_empty());
}
