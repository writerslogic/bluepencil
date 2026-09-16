use std::collections::HashSet;
use std::sync::OnceLock;

use crate::lexicon::embedded;

fn set() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| embedded::lines(embedded::ABBREVIATIONS).collect())
}

/// True when the token (without its trailing period) is a known abbreviation or an initial.
pub fn is_abbreviation(token: &str) -> bool {
    let token = token.trim_start_matches(|c: char| !c.is_alphanumeric());
    if token.is_empty() {
        return false;
    }
    let mut chars = token.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        return c.is_uppercase();
    }
    if token.contains('.') {
        return true;
    }
    set().contains(token.to_lowercase().as_str())
}
