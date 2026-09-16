use super::{Mask, lines};

/// Blanks a leading YAML (`---`) or TOML (`+++`) block and returns where the body starts.
pub fn strip(source: &str, mask: &mut Mask) -> usize {
    let mut it = lines(source);
    let Some((_, first)) = it.next() else {
        return 0;
    };
    let delim = first.trim_end();
    if delim != "---" && delim != "+++" {
        return 0;
    }
    for (start, line) in it {
        if line.trim_end() == delim || (delim == "---" && line.trim_end() == "...") {
            let end = start + line.len();
            mask.blank(0, end);
            return end;
        }
    }
    0
}
