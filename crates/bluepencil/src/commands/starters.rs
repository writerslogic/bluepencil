use anyhow::Result;
use bluepencil_core::analysis::starters::starters;

use crate::cli::StartersArgs;
use crate::context::Context;
use crate::output::human::{file_banner, findings, heading, ranked};
use crate::output::{json, locate};

pub fn run(ctx: &Context, args: &StartersArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let run = args.run.unwrap_or(ctx.config.starters.run).max(2);
    let results: Vec<_> = docs.iter().map(|d| starters(d, run)).collect();

    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .zip(&results)
            .map(|(d, s)| {
                let f: Vec<_> = s.findings.iter().map(|f| locate(d, f)).collect();
                serde_json::json!({ "file": d.name, "sentence": s.sentence, "paragraph": s.paragraph, "findings": f })
            })
            .collect();
        return json::print(&out);
    }

    for (doc, s) in docs.iter().zip(&results) {
        file_banner(&docs, doc);
        let n = doc.sentences().count();
        let top: Vec<(String, usize)> = s
            .sentence
            .iter()
            .map(|(w, c)| (format!("{w} ({:.0}%)", *c as f64 * 100.0 / n.max(1) as f64), *c))
            .collect();
        heading("Sentence openers");
        ranked(&top, "word", ctx.limit.min(15), None);
        heading("Paragraph openers");
        ranked(&s.paragraph, "word", ctx.limit.min(10), None);
        heading(&format!("Runs of {run}+ sentences with the same opener"));
        let items: Vec<_> = s.findings.iter().map(|f| (doc, f)).collect();
        findings(&items);
    }
    Ok(())
}
