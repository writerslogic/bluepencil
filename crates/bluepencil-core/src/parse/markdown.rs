use super::{Mask, Parsed, frontmatter, lines};
use crate::document::Heading;
use crate::span::Span;

pub fn parse(source: &str) -> Parsed {
    let mut mask = Mask::new(source);
    let body = frontmatter::strip(source, &mut mask);
    let mut headings = Vec::new();
    let mut breaks = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut in_comment = false;
    let mut prev_text: Option<(usize, &str)> = None;

    for (start, line) in lines(source) {
        if start < body {
            continue;
        }
        let end = start + line.len();

        if in_comment {
            match line.find("-->") {
                Some(i) => {
                    mask.blank(start, start + i + 3);
                    in_comment = false;
                    inline(source, &mut mask, start + i + 3, end);
                }
                None => mask.blank(start, end),
            }
            continue;
        }

        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if let Some((ch, len)) = fence {
            mask.blank(start, end);
            let run = trimmed.bytes().take_while(|&b| b == ch).count();
            if run >= len && trimmed[run..].trim().is_empty() {
                fence = None;
            }
            prev_text = None;
            continue;
        }

        if let Some(ch @ (b'`' | b'~')) = trimmed.bytes().next() {
            let run = trimmed.bytes().take_while(|&b| b == ch).count();
            if run >= 3 {
                fence = Some((ch, run));
                mask.blank(start, end);
                prev_text = None;
                continue;
            }
        }

        if trimmed.is_empty() {
            prev_text = None;
            continue;
        }

        if let Some(level) = atx_level(trimmed) {
            let title = trimmed[level..].trim().trim_end_matches('#').trim();
            headings.push(heading(level, title, start + indent, end));
            mask.blank(start, end);
            prev_text = None;
            continue;
        }

        if let Some(level) = setext_level(trimmed) {
            if let Some((pstart, ptext)) = prev_text {
                headings.push(heading(level, ptext.trim(), pstart, pstart + ptext.len()));
                mask.blank(pstart, pstart + ptext.len());
            }
            mask.blank(start, end);
            prev_text = None;
            continue;
        }

        if is_thematic_break(trimmed) || trimmed.starts_with('|') {
            mask.blank(start, end);
            prev_text = None;
            continue;
        }

        let mut content = start + indent;
        loop {
            let rest = &source[content..end];
            if let Some(after) = rest.strip_prefix('>') {
                let skip = 1 + (after.len() - after.trim_start().len());
                mask.blank(content, content + skip);
                content += skip;
                continue;
            }
            if let Some(marker) = list_marker(rest) {
                mask.blank(content, content + marker);
                breaks.push(content + marker);
                content += marker;
                continue;
            }
            break;
        }

        if let Some(i) = source[content..end].find("<!--") {
            let open = content + i;
            match source[open..end].find("-->") {
                Some(j) => {
                    mask.blank(open, open + j + 3);
                    inline(source, &mut mask, content, open);
                    inline(source, &mut mask, open + j + 3, end);
                }
                None => {
                    mask.blank(open, end);
                    in_comment = true;
                    inline(source, &mut mask, content, open);
                }
            }
        } else {
            inline(source, &mut mask, content, end);
        }
        prev_text = Some((start, line));
    }

    Parsed { prose: mask.finish(), headings, breaks, dialogue: None }
}

fn heading(level: usize, title: &str, start: usize, end: usize) -> Heading {
    Heading { level: level as u8, title: strip_inline(title), span: Span::new(start, end) }
}

fn strip_inline(title: &str) -> String {
    let mut mask = Mask::new(title);
    inline(title, &mut mask, 0, title.len());
    mask.finish().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn atx_level(line: &str) -> Option<usize> {
    let level = line.bytes().take_while(|&b| b == b'#').count();
    let after = line.as_bytes().get(level);
    ((1..=6).contains(&level) && matches!(after, None | Some(b' ' | b'\t'))).then_some(level)
}

fn setext_level(line: &str) -> Option<usize> {
    let t = line.trim_end();
    if !t.is_empty() && t.bytes().all(|b| b == b'=') {
        Some(1)
    } else if t.len() >= 2 && t.bytes().all(|b| b == b'-') {
        Some(2)
    } else {
        None
    }
}

fn is_thematic_break(line: &str) -> bool {
    let chars: Vec<u8> = line.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    chars.len() >= 3 && matches!(chars[0], b'*' | b'-' | b'_') && chars.iter().all(|&b| b == chars[0])
}

fn list_marker(rest: &str) -> Option<usize> {
    let b = rest.as_bytes();
    let spaced = |i: usize| matches!(b.get(i), Some(b' ' | b'\t'));
    if matches!(b.first(), Some(b'-' | b'*' | b'+')) && spaced(1) {
        let mut n = 2;
        if rest[n..].starts_with("[ ] ") || rest[n..].starts_with("[x] ") || rest[n..].starts_with("[X] ") {
            n += 4;
        }
        return Some(n);
    }
    let digits = b.iter().take_while(|c| c.is_ascii_digit()).count();
    if (1..=9).contains(&digits) && matches!(b.get(digits), Some(b'.' | b')')) && spaced(digits + 1) {
        return Some(digits + 2);
    }
    None
}

fn inline(source: &str, mask: &mut Mask, start: usize, end: usize) {
    let b = source.as_bytes();
    let find = |from: usize, pat: &str| source[from..end].find(pat).map(|i| from + i);
    let mut i = start;
    while i < end {
        match b[i] {
            b'`' => {
                let run = b[i..end].iter().take_while(|&&c| c == b'`').count();
                let ticks = &source[i..i + run];
                match find(i + run, ticks) {
                    Some(close) => {
                        mask.blank(i, close + run);
                        i = close + run;
                    }
                    None => {
                        mask.blank(i, i + run);
                        i += run;
                    }
                }
                continue;
            }
            b'!' if b.get(i + 1) == Some(&b'[') => {
                if let Some(close) = find(i, "](").and_then(|m| find(m, ")")) {
                    mask.blank(i, close + 1);
                    i = close + 1;
                    continue;
                }
            }
            b'[' => {
                if b.get(i + 1) == Some(&b'^')
                    && let Some(close) = find(i, "]")
                {
                    mask.blank(i, close + 1);
                    i = close + 1;
                    continue;
                }
                if let Some(close) = find(i + 1, "]") {
                    match b.get(close + 1) {
                        Some(b'(') => {
                            if let Some(paren) = find(close, ")") {
                                mask.blank(i, i + 1);
                                mask.blank(close, paren + 1);
                                i = paren + 1;
                                continue;
                            }
                        }
                        Some(b'[') => {
                            if let Some(r) = find(close + 1, "]") {
                                mask.blank(i, i + 1);
                                mask.blank(close, r + 1);
                                i = r + 1;
                                continue;
                            }
                        }
                        _ => {}
                    }
                }
            }
            b'<' if b.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic() || *c == b'/') => {
                if let Some(close) = find(i, ">") {
                    mask.blank(i, close + 1);
                    i = close + 1;
                    continue;
                }
            }
            b'h' if source[i..end].starts_with("http://") || source[i..end].starts_with("https://") => {
                let stop = source[i..end].find(char::is_whitespace).map_or(end, |n| i + n);
                mask.blank(i, stop);
                i = stop;
                continue;
            }
            b'\\' if b.get(i + 1).is_some_and(u8::is_ascii_punctuation) => {
                mask.blank(i, i + 1);
                i += 2;
                continue;
            }
            b'*' => mask.blank(i, i + 1),
            b'~' if b.get(i + 1) == Some(&b'~') => {
                mask.blank(i, i + 2);
                i += 2;
                continue;
            }
            b'_' => {
                let before = i > start && b[i - 1].is_ascii_alphanumeric();
                let after = b.get(i + 1).is_some_and(u8::is_ascii_alphanumeric);
                if !(before && after) {
                    mask.blank(i, i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
}
