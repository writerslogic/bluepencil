use bluepencil_core::analysis::repetition::repeats;
use bluepencil_core::{Document, Format, Lexicons};

fn phrases(text: &str, min: usize, max: usize, min_count: usize) -> Vec<String> {
    let lex = Lexicons::default();
    let doc = Document::parse("t", text, Format::Plain);
    repeats(&doc, &lex, min, max, min_count).into_iter().map(|r| r.phrase).collect()
}

#[test]
fn shorter_phrase_fully_inside_a_longer_repeat_is_dropped() {
    let text = "the dark forest at midnight. the dark forest at midnight. the dark forest at midnight.";
    let found = phrases(text, 2, 4, 2);
    assert!(found.contains(&"the dark forest at".to_string()));
    assert!(!found.contains(&"dark forest".to_string()));
}

#[test]
fn shorter_phrase_with_enough_uncovered_occurrences_survives() {
    // "dark forest" is fully covered by the longer 4-gram in the first two sentences,
    // but occurs twice more on its own -- two uncovered hits meets min_count on its own.
    let text = "the dark forest at midnight. the dark forest at midnight. \
                a dark forest loomed. another dark forest appeared.";
    let found = phrases(text, 2, 4, 2);
    assert!(found.contains(&"dark forest".to_string()));
}

#[test]
fn below_min_count_is_not_reported() {
    let found = phrases("the dark forest at midnight. nothing else repeats here at all.", 2, 4, 2);
    assert!(found.is_empty());
}
