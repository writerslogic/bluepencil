use bluepencil_core::analysis::diff::{ChangeKind, word_diff};

fn words(diff: &bluepencil_core::analysis::diff::WordDiff, tag: ChangeKind) -> Vec<&str> {
    diff.changes.iter().filter(|c| c.tag == tag).map(|c| c.word.as_str()).collect()
}

#[test]
fn identical_text_has_no_changes() {
    let diff = word_diff("the quick brown fox", "the quick brown fox");
    assert_eq!(diff.added, 0);
    assert_eq!(diff.removed, 0);
}

#[test]
fn detects_an_inserted_word() {
    let diff = word_diff("the quick fox", "the quick brown fox");
    assert_eq!(diff.added, 1);
    assert_eq!(diff.removed, 0);
    assert_eq!(words(&diff, ChangeKind::Insert), vec!["brown"]);
}

#[test]
fn detects_a_removed_word() {
    let diff = word_diff("the quick brown fox", "the quick fox");
    assert_eq!(diff.added, 0);
    assert_eq!(diff.removed, 1);
    assert_eq!(words(&diff, ChangeKind::Delete), vec!["brown"]);
}

#[test]
fn punctuation_stays_attached_to_its_word() {
    let diff = word_diff("She said hello.", "She said hello, warmly.");
    assert_eq!(diff.removed, 1);
    assert_eq!(diff.added, 2);
    assert_eq!(words(&diff, ChangeKind::Delete), vec!["hello."]);
}
