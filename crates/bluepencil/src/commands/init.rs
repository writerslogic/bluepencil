use anyhow::{Result, bail};

use crate::config::{FILE_NAME, TEMPLATE};
use crate::exit::Status;

pub fn run(force: bool) -> Result<Status> {
    let path = std::path::Path::new(FILE_NAME);
    if path.exists() && !force {
        bail!("{FILE_NAME} already exists (use --force to overwrite)");
    }
    std::fs::write(path, TEMPLATE)?;
    println!("Wrote {FILE_NAME}");
    Ok(Status::Ok)
}
