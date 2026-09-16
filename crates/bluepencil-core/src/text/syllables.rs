use std::collections::HashMap;
use std::sync::OnceLock;

use crate::lexicon::embedded;

fn exceptions() -> &'static HashMap<&'static str, usize> {
    static MAP: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    MAP.get_or_init(|| {
        embedded::lines(embedded::SYLLABLE_EXCEPTIONS)
            .filter_map(|l| l.split_once('\t'))
            .filter_map(|(w, n)| Some((w, n.trim().parse().ok()?)))
            .collect()
    })
}

/// Heuristic English syllable count; exact enough for readability formulas.
pub fn count(word: &str) -> usize {
    let lower = word.to_lowercase();
    if let Some(&n) = exceptions().get(lower.as_str()) {
        return n;
    }
    let mut w: String = lower.chars().filter(char::is_ascii_alphabetic).collect();
    if w.is_empty() {
        return 1;
    }
    if w.len() <= 3 {
        return 1;
    }
    let silent_es =
        w.ends_with("es") && !["ses", "zes", "ces", "ges", "xes", "shes", "ches"].iter().any(|s| w.ends_with(s));
    let silent_ed = w.ends_with("ed") && !w.ends_with("ted") && !w.ends_with("ded");
    if silent_es || silent_ed {
        w.truncate(w.len() - 2);
    } else if w.ends_with('e') && !w.ends_with("le") && !w.ends_with("ee") {
        w.pop();
    }
    let vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
    let mut groups = 0;
    let mut prev = false;
    for (i, c) in w.chars().enumerate() {
        let v = vowel(c) && !(c == 'y' && i == 0);
        if v && !prev {
            groups += 1;
        }
        prev = v;
    }
    groups.max(1)
}
