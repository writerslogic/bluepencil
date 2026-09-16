/// Light stemmer so echoes catch `door`/`doors` and `city`/`cities` without over-merging.
pub fn stem(word: &str) -> String {
    let w = word.strip_suffix("'s").unwrap_or(word);
    if w.len() <= 3 {
        return w.to_string();
    }
    if let Some(base) = w.strip_suffix("ies") {
        return format!("{base}y");
    }
    if let Some(base) = w.strip_suffix("sses") {
        return format!("{base}ss");
    }
    for suffix in ["ches", "shes", "xes"] {
        if w.ends_with(suffix) {
            return w[..w.len() - 2].to_string();
        }
    }
    if w.ends_with('s') && !w.ends_with("ss") && !w.ends_with("us") && !w.ends_with("is") {
        return w[..w.len() - 1].to_string();
    }
    w.to_string()
}
