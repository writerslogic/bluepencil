use bluepencil_core::analysis::overuse::overused;
use bluepencil_core::{Document, Format, Lexicons};

fn lexicons_with_baseline(baseline: &[(&str, f64)]) -> Lexicons {
    Lexicons { english_frequency: baseline.iter().map(|(w, f)| (w.to_string(), *f)).collect(), ..Lexicons::default() }
}

#[test]
fn word_far_above_baseline_is_flagged() {
    // "crimson" appears 4 times in ~10 words (400,000 per million); a baseline of 5
    // per million makes that an 80,000x ratio, comfortably above any min_ratio.
    let lex = lexicons_with_baseline(&[("crimson", 5.0), ("sky", 500.0)]);
    let doc = Document::parse(
        "t",
        "The crimson sky. Crimson clouds drifted. A crimson bird flew. Crimson everywhere.",
        Format::Plain,
    );
    let rows = overused(&[doc], &lex, 3.0, 3);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].word, "crimson");
    assert_eq!(rows[0].count, 4);
}

#[test]
fn word_absent_from_baseline_is_skipped() {
    let lex = lexicons_with_baseline(&[("sky", 500.0)]);
    let doc = Document::parse("t", "Zorbnak zorbnak zorbnak zorbnak.", Format::Plain);
    assert!(overused(&[doc], &lex, 3.0, 3).is_empty());
}

#[test]
fn below_min_count_is_skipped_even_with_huge_ratio() {
    let lex = lexicons_with_baseline(&[("crimson", 0.001)]);
    let doc = Document::parse("t", "The crimson sky over the crimson hills.", Format::Plain);
    // "crimson" occurs twice, below the default min_count of 3.
    assert!(overused(&[doc], &lex, 3.0, 3).is_empty());
}

#[test]
fn empty_baseline_produces_no_findings() {
    let lex = lexicons_with_baseline(&[]);
    let doc = Document::parse("t", "Crimson crimson crimson crimson.", Format::Plain);
    assert!(overused(&[doc], &lex, 3.0, 1).is_empty());
}

#[test]
fn real_bundled_corpus_flags_a_hammered_word_but_not_ordinary_prose() {
    let lex = Lexicons::default();
    assert!(!lex.english_frequency.is_empty(), "bundled corpus should be populated");
    let hammered = Document::parse(
        "t",
        "The kettle sat on the stove. She filled the kettle again. \
         The kettle whistled, and she poured from the kettle once more.",
        Format::Plain,
    );
    let rows = overused(&[hammered], &lex, 3.0, 3);
    assert!(rows.iter().any(|r| r.word == "kettle"), "{rows:?}");

    let ordinary = Document::parse(
        "t",
        "She walked into the kitchen and made a cup of coffee before the meeting started.",
        Format::Plain,
    );
    assert!(overused(&[ordinary], &lex, 3.0, 3).is_empty());
}
