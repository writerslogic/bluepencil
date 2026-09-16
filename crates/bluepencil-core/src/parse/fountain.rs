use super::{Mask, Parsed, lines};
use crate::document::Heading;
use crate::span::Span;

const SCENE_PREFIXES: [&str; 6] = ["INT", "EXT", "EST", "INT./EXT", "INT/EXT", "I/E"];

pub fn parse(source: &str) -> Parsed {
    let mut mask = Mask::new(source);
    let all: Vec<(usize, &str)> = lines(source).collect();
    let mut headings = Vec::new();
    let mut dialogue = Vec::new();
    let mut i = title_page(&all, &mut mask);
    let mut in_dialogue = false;

    while i < all.len() {
        let (start, line) = all[i];
        let end = start + line.len();
        let t = line.trim();
        let prev_blank = i == 0 || all[i - 1].1.trim().is_empty();
        let next_text = all.get(i + 1).is_some_and(|(_, l)| !l.trim().is_empty());
        i += 1;

        if t.is_empty() {
            in_dialogue = false;
            continue;
        }

        if in_dialogue {
            if t.starts_with('(') && t.ends_with(')') {
                mask.blank(start, end);
            } else {
                let lead = line.len() - line.trim_start().len();
                dialogue.push(Span::new(start + lead, start + lead + t.len()));
                emphasis(source, &mut mask, start, end);
            }
            continue;
        }

        if t.starts_with("[[") || t.starts_with("/*") || t.starts_with('=') {
            mask.blank(start, end);
            continue;
        }

        if let Some(rest) = t.strip_prefix('#') {
            let level = 1 + rest.bytes().take_while(|&b| b == b'#').count();
            let title = t.trim_start_matches('#').trim().to_string();
            headings.push(Heading { level: level.min(6) as u8, title, span: Span::new(start, end) });
            mask.blank(start, end);
            continue;
        }

        if prev_blank && is_scene_heading(t) {
            let title = t.trim_start_matches('.').trim().to_string();
            headings.push(Heading { level: 2, title, span: Span::new(start, end) });
            mask.blank(start, end);
            continue;
        }

        if is_transition(t) {
            mask.blank(start, end);
            continue;
        }

        if prev_blank && next_text && is_character_cue(t) {
            mask.blank(start, end);
            in_dialogue = true;
            continue;
        }

        if t.len() >= 2 && t.starts_with('>') && t.ends_with('<') {
            let lead = start + (line.len() - line.trim_start().len());
            mask.blank(lead, lead + 1);
            mask.blank(lead + t.len() - 1, lead + t.len());
        } else if t.starts_with('!') {
            let lead = start + (line.len() - line.trim_start().len());
            mask.blank(lead, lead + 1);
        }
        emphasis(source, &mut mask, start, end);
    }

    blank_between(source, &mut mask, "[[", "]]");
    blank_between(source, &mut mask, "/*", "*/");
    dialogue.sort();
    Parsed { prose: mask.finish(), headings, breaks: Vec::new(), dialogue: Some(dialogue) }
}

fn title_page(all: &[(usize, &str)], mask: &mut Mask) -> usize {
    let is_key = |l: &str| {
        l.split_once(':').is_some_and(|(k, _)| !k.is_empty() && k.chars().all(|c| c.is_alphabetic() || c == ' '))
    };
    if !all.first().is_some_and(|(_, l)| is_key(l)) {
        return 0;
    }
    let mut i = 0;
    while i < all.len() && !all[i].1.trim().is_empty() {
        let (start, line) = all[i];
        mask.blank(start, start + line.len());
        i += 1;
    }
    i
}

fn is_scene_heading(t: &str) -> bool {
    if t.starts_with('.') && !t.starts_with("..") {
        return true;
    }
    let upper = t.to_ascii_uppercase();
    SCENE_PREFIXES.iter().any(|p| upper.strip_prefix(p).is_some_and(|r| r.starts_with('.') || r.starts_with(' ')))
}

fn is_transition(t: &str) -> bool {
    (t.starts_with('>') && !t.ends_with('<'))
        || (t.ends_with("TO:") && t == t.to_uppercase() && t.chars().any(char::is_alphabetic))
}

fn is_character_cue(t: &str) -> bool {
    if let Some(rest) = t.strip_prefix('@') {
        return !rest.trim().is_empty();
    }
    let name = t.split('(').next().unwrap_or(t).trim_end_matches('^').trim();
    name.chars().any(char::is_alphabetic) && name.chars().all(|c| !c.is_lowercase()) && !name.ends_with(':')
}

fn emphasis(source: &str, mask: &mut Mask, start: usize, end: usize) {
    for (i, b) in source.as_bytes()[start..end].iter().enumerate() {
        if matches!(b, b'*' | b'_') {
            mask.blank(start + i, start + i + 1);
        }
    }
}

fn blank_between(source: &str, mask: &mut Mask, open: &str, close: &str) {
    let mut from = 0;
    while let Some(i) = source[from..].find(open) {
        let s = from + i;
        let e = source[s..].find(close).map_or(source.len(), |j| s + j + close.len());
        mask.blank(s, e);
        from = e;
    }
}
