use std::io;
use std::path::Path;

/// Reads a user word list: one entry per line, `#` comments allowed.
pub fn read_list(path: &Path) -> io::Result<Vec<String>> {
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(crate::text::normalize::fold)
        .collect())
}
