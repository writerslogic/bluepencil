use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

/// Expands patterns relative to `base`, keeping order and dropping duplicates.
pub fn expand(patterns: &[String], base: &Path) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = Vec::new();
    for pattern in patterns {
        let full = if Path::new(pattern).is_absolute() { PathBuf::from(pattern) } else { base.join(pattern) };
        let mut matched: Vec<PathBuf> = match ::glob::glob(&full.to_string_lossy()) {
            Ok(paths) => paths.filter_map(Result::ok).filter(|p| p.is_file()).collect(),
            Err(_) => Vec::new(),
        };
        if matched.is_empty() && full.is_file() {
            matched.push(full);
        }
        if matched.is_empty() {
            bail!("no files match `{pattern}`");
        }
        matched.sort();
        for p in matched {
            if !out.contains(&p) {
                out.push(p);
            }
        }
    }
    Ok(out)
}

/// Shortens a path for display when it sits under the working directory.
pub fn display(path: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    path.strip_prefix(&cwd).unwrap_or(path).display().to_string()
}
