use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::cli::{Input, SplitArgs};
use crate::context::Context;

fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut last_was_dash = true; // suppress a leading dash
    for c in title.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() { "untitled".to_string() } else { out }
}

pub fn run(ctx: &Context, args: &SplitArgs) -> Result<()> {
    let docs = ctx.documents(&Input { files: vec![args.path.clone()] })?;
    let [doc] = docs.as_slice() else {
        bail!("split takes exactly one file, matched {}", docs.len());
    };

    let headings: Vec<_> = doc.headings.iter().filter(|h| h.level == args.level).collect();
    if headings.is_empty() {
        bail!("no level-{} headings found in {}", args.level, args.path);
    }

    let source_path = PathBuf::from(&args.path);
    let ext = source_path.extension().and_then(|e| e.to_str()).unwrap_or("txt");
    let out_dir = args.out.clone().unwrap_or_else(|| {
        source_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(source_path.file_stem().and_then(|s| s.to_str()).unwrap_or("split"))
    });
    std::fs::create_dir_all(&out_dir)?;

    let width = headings.len().to_string().len();
    for (i, heading) in headings.iter().enumerate() {
        let end = headings.get(i + 1).map_or(doc.source.len(), |h| h.span.start);
        let chunk = &doc.source[heading.span.start..end];
        let filename = format!("{:0width$}-{}.{ext}", i + 1, slug(&heading.title), width = width);
        let path = out_dir.join(&filename);
        std::fs::write(&path, chunk)?;
        println!("{}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::slug;

    #[test]
    fn punctuation_becomes_single_dashes() {
        assert_eq!(slug("Chapter Two: The Middle!"), "chapter-two-the-middle");
    }

    #[test]
    fn leading_and_trailing_punctuation_is_dropped() {
        assert_eq!(slug("  -- Foo --  "), "foo");
    }

    #[test]
    fn punctuation_only_title_falls_back_to_untitled() {
        assert_eq!(slug("***"), "untitled");
    }

    #[test]
    fn empty_title_falls_back_to_untitled() {
        assert_eq!(slug(""), "untitled");
    }
}
