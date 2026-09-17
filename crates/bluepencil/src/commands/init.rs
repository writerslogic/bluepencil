use anyhow::{Result, bail};

use crate::cli::Genre;
use crate::config::{FILE_NAME, TEMPLATE};
use crate::exit::Status;

/// Curated by hand alongside `examples/<genre>/bluepencil.toml`, which stays the source a
/// human reads and edits; these are `include_str!`'d copies so `init --genre` writes exactly
/// what's checked in there rather than a second, driftable copy of the same settings.
fn template_for(genre: Genre) -> &'static str {
    match genre {
        Genre::Novel => include_str!("../../../../examples/novel/bluepencil.toml"),
        Genre::Essay => include_str!("../../../../examples/essay/bluepencil.toml"),
        Genre::Screenplay => include_str!("../../../../examples/screenplay/bluepencil.toml"),
    }
}

pub fn run(force: bool, genre: Option<Genre>) -> Result<Status> {
    let path = std::path::Path::new(FILE_NAME);
    if path.exists() && !force {
        bail!("{FILE_NAME} already exists (use --force to overwrite)");
    }
    let contents = genre.map_or(TEMPLATE, template_for);
    std::fs::write(path, contents)?;
    println!("Wrote {FILE_NAME}");
    Ok(Status::Ok)
}
