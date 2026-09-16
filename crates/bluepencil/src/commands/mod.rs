mod adverbs;
mod check;
mod cliches;
mod completions;
mod count;
mod dialogue;
mod echoes;
mod filter;
mod freq;
mod hapax;
mod hedges;
mod histogram;
mod init;
mod outline;
mod readability;
mod repeats;
pub mod report;
mod rhythm;
mod starters;
mod tics;
mod unique;

use anyhow::Result;
use bluepencil_core::analysis::tally;
use bluepencil_core::{Document, Finding, Lexicons};

use crate::cli::{Cli, Command, ListArgs};
use crate::context::Context;
use crate::exit::Status;
use crate::output::{self, human};

pub fn run(cli: Cli) -> Result<Status> {
    match cli.command {
        Command::Init { force } => return init::run(force),
        Command::Completions { shell } => return completions::run(shell),
        _ => {}
    }
    let ctx = Context::new(&cli.global)?;
    match cli.command {
        Command::Count(a) => count::run(&ctx, &a),
        Command::Outline(a) => outline::run(&ctx, &a),
        Command::Histogram(a) => histogram::run(&ctx, &a),
        Command::Rhythm(a) => rhythm::run(&ctx, &a),
        Command::Freq(a) => freq::run(&ctx, &a),
        Command::Unique(a) => unique::run(&ctx, &a),
        Command::Hapax(a) => hapax::run(&ctx, &a),
        Command::Repeats(a) => repeats::run(&ctx, &a),
        Command::Echoes(a) => echoes::run(&ctx, &a),
        Command::Starters(a) => starters::run(&ctx, &a),
        Command::Tics(a) => tics::run(&ctx, &a),
        Command::Filter(a) => filter::run(&ctx, &a),
        Command::Hedges(a) => hedges::run(&ctx, &a),
        Command::Adverbs(a) => adverbs::run(&ctx, &a),
        Command::Cliches(a) => cliches::run(&ctx, &a),
        Command::Readability(a) => readability::run(&ctx, &a),
        Command::Dialogue(a) => dialogue::run(&ctx, &a),
        Command::Report(a) => report::run(&ctx, &a),
        Command::Check(a) => return check::run(&ctx, &a),
        Command::Init { .. } | Command::Completions { .. } => unreachable!(),
    }?;
    Ok(Status::Ok)
}

type Finder = fn(&Document, &Lexicons) -> Vec<Finding>;

/// Shared driver for commands that locate words or phrases from a list.
pub(crate) fn list_command(ctx: &Context, args: &ListArgs, find: Finder, label: &str) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let per_doc: Vec<Vec<Finding>> = docs.iter().map(|d| find(d, &ctx.lexicons)).collect();
    let total_words: usize = docs.iter().map(Document::word_count).sum();
    let all: Vec<Finding> = per_doc.iter().flatten().cloned().collect();

    if ctx.json {
        let located: Vec<_> =
            docs.iter().zip(&per_doc).flat_map(|(d, fs)| fs.iter().map(|f| output::locate(d, f))).collect();
        return output::json::print(&serde_json::json!({
            "total": all.len(),
            "per_1k_words": bluepencil_core::stats::per_thousand(all.len(), total_words),
            "summary": tally(&all),
            "findings": located,
        }));
    }

    if args.summary {
        human::ranked(&tally(&all), label, ctx.limit, Some(total_words));
    } else {
        let items: Vec<_> = docs.iter().zip(&per_doc).flat_map(|(d, fs)| fs.iter().map(move |f| (d, f))).collect();
        human::findings(&items);
    }
    println!(
        "\n{} found, {:.1} per 1,000 words",
        all.len(),
        bluepencil_core::stats::per_thousand(all.len(), total_words)
    );
    Ok(())
}
