use bluepencil_core::{Document, Format};

fn sentences(text: &str) -> Vec<String> {
    let doc = Document::parse("t", text, Format::Plain);
    doc.sentences().map(|s| doc.text(s.span).to_string()).collect()
}

#[test]
fn abbreviations_and_initials_do_not_split() {
    assert_eq!(
        sentences("Mr. Hale met Dr. Ames in the U.S. Army. J. R. R. Tolkien wrote it at 5 p.m. on Friday. Done."),
        ["Mr. Hale met Dr. Ames in the U.S. Army.", "J. R. R. Tolkien wrote it at 5 p.m. on Friday.", "Done.",]
    );
}

#[test]
fn dialogue_punctuation_before_lowercase_tag_does_not_split() {
    assert_eq!(
        sentences("\u{201c}Why?\u{201d} she asked. \u{201c}Because.\u{201d} He left."),
        ["\u{201c}Why?\u{201d} she asked.", "\u{201c}Because.\u{201d}", "He left."]
    );
}

#[test]
fn ellipsis_splits_only_before_a_capital() {
    assert_eq!(
        sentences("It was late... and cold... Then it ended."),
        ["It was late... and cold...", "Then it ended."]
    );
}

#[test]
fn markdown_markup_is_ignored_but_offsets_survive() {
    let src = "---\ntitle: x\n---\n# Heading\n\nShe saw *it*. `code here` Then [a link](http://x.y) ended.\n\n```\nnot prose.\n```\n";
    let doc = Document::parse("t.md", src, Format::Markdown);
    let got: Vec<&str> = doc.sentences().map(|s| doc.text(s.span)).collect();
    assert_eq!(got, ["She saw *it*.", "Then [a link](http://x.y) ended."]);
    assert_eq!(doc.headings[0].title, "Heading");
    assert_eq!(doc.word_count(), 7);
}
