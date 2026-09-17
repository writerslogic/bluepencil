use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_dir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("bluepencil-check-since-test-{nanos}-{}-{n}", std::process::id()))
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .status()
        .expect("running git");
    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

#[test]
fn scopes_to_changed_lines_when_run_from_an_unrelated_subdirectory() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    let other = dir.join("other");
    std::fs::create_dir_all(&chapters).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);

    // `project.files` is expanded relative to the config's directory (the repo root here),
    // not the current one, so running from `other/` surfaces an absolute `doc.name`. That
    // absolute filesystem path must still resolve against the repo root the same way `git
    // diff` expects, or the changed-line lookup silently comes back empty and the violation
    // in the new cliche never gets reported.
    std::fs::write(dir.join("bluepencil.toml"), "project.files = [\"chapters/*.md\"]\n\n[check]\nmax_cliches = 0\n")
        .unwrap();
    std::fs::write(chapters.join("one.md"), "One two three.\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "first"]);

    std::fs::write(chapters.join("one.md"), "One two three. At the end of the day, four.\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "second"]);

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&other)
        .args(["--json", "check", "--since", "HEAD~1"])
        .output()
        .expect("running bluepencil");

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["passed"], false, "expected the new cliche on the changed line to fail the check: {json}");
    assert!(!output.status.success());

    // The `:/` top-anchor must be a no-op from the repo root itself.
    let root_output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "check", "--since", "HEAD~1"])
        .output()
        .expect("running bluepencil");
    let root_json: serde_json::Value = serde_json::from_slice(&root_output.stdout).unwrap();
    assert_eq!(root_json["passed"], false, "root run must also flag the cliche: {root_json}");
    assert_eq!(root_json["violations"][0]["rule"], json["violations"][0]["rule"]);

    std::fs::remove_dir_all(&dir).ok();
}
