use bluepencil_core::{Document, Finding};

use super::table::{Align, Table};
use super::{excerpt, location};

pub fn heading(title: &str) {
    println!("\n{title}");
}

/// Prints a file banner when more than one document is being reported.
pub fn file_banner(docs: &[Document], doc: &Document) {
    if docs.len() > 1 {
        println!("\n== {} ==", doc.name);
    }
}

pub fn findings(items: &[(&Document, &Finding)]) {
    if items.is_empty() {
        println!("Nothing found.");
        return;
    }
    for (doc, f) in items {
        println!("{}  {}  {}", location(doc, f.span), f.rule, f.message);
        let text = excerpt(doc, f.span, 90);
        if text != f.message {
            println!("    {text}");
        }
    }
}

pub fn ranked(rows: &[(String, usize)], label: &str, limit: usize, total_words: Option<usize>) {
    if rows.is_empty() {
        println!("Nothing found.");
        return;
    }
    let mut headers = vec![(label, Align::Left), ("count", Align::Right)];
    if total_words.is_some() {
        headers.push(("per 1k", Align::Right));
    }
    let mut t = Table::new(&headers);
    for (word, n) in rows.iter().take(limit) {
        let mut row = vec![word.clone(), n.to_string()];
        if let Some(total) = total_words {
            row.push(format!("{:.1}", bluepencil_core::stats::per_thousand(*n, total)));
        }
        t.row(row);
    }
    t.print();
    if rows.len() > limit {
        println!("… {} more (use -n to show more)", rows.len() - limit);
    }
}

pub fn minutes(m: f64) -> String {
    let total = m.round() as u64;
    if total < 60 { format!("{total} min") } else { format!("{}h {:02}m", total / 60, total % 60) }
}

pub fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}
