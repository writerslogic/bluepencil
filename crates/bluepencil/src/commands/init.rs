use anyhow::{Result, bail};

use crate::cli::Genre;
use crate::config::{FILE_NAME, TEMPLATE};
use crate::exit::Status;

/// The presets live in `templates/` so they ship inside the published crate; the example
/// projects under `examples/<genre>/` carry byte-identical copies, and `tests/init.rs` fails
/// if the two drift.
fn template_for(genre: Genre) -> &'static str {
    match genre {
        Genre::Novel => include_str!("../../templates/novel.toml"),
        Genre::Essay => include_str!("../../templates/essay.toml"),
        Genre::Screenplay => include_str!("../../templates/screenplay.toml"),
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
