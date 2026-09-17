use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_dir() -> PathBuf {
    // Tests run in parallel threads of one process; SystemTime alone can collide when clock
    // resolution is coarser than thread scheduling, so an atomic counter guarantees uniqueness.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("bluepencil-progress-test-{nanos}-{}-{n}", std::process::id()))
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
fn reports_word_count_change_since_a_commit() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);

    std::fs::write(chapters.join("one.md"), "One two three four five.\n").unwrap();
    git(&dir, &["add", "chapters/one.md"]);
    git(&dir, &["commit", "-q", "-m", "first"]);

    std::fs::write(chapters.join("one.md"), "One two three four five six seven eight nine ten.\n").unwrap();
    git(&dir, &["add", "chapters/one.md"]);
    git(&dir, &["commit", "-q", "-m", "second"]);

    // A glob pattern rather than a literal filename: on Windows, `glob::glob` returns
    // native-separator (`\`) PathBufs, which is what actually exercised the bug where
    // progress compared a filesystem path against a git-relative one.
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "progress", "--since", "HEAD~1", "chapters/*.md"])
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total_before"], 5);
    assert_eq!(json["total_after"], 10);
    assert_eq!(json["total_change"], 5);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn reports_zero_before_for_a_file_that_did_not_exist_yet() {
    let dir = unique_dir();
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);

    std::fs::write(dir.join(".gitkeep"), "").unwrap();
    git(&dir, &["add", ".gitkeep"]);
    git(&dir, &["commit", "-q", "-m", "empty"]);

    std::fs::write(dir.join("chapter.md"), "One two three.\n").unwrap();
    git(&dir, &["add", "chapter.md"]);
    git(&dir, &["commit", "-q", "-m", "add chapter"]);

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "progress", "--since", "HEAD~1", "chapter.md"])
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total_before"], 0);
    assert_eq!(json["total_after"], 3);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn resolves_project_files_when_run_from_an_unrelated_subdirectory() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    let other = dir.join("other");
    std::fs::create_dir_all(&chapters).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);

    // `project.files` is expanded relative to the config's directory (the repo root here),
    // not the current one, so running from `other/` (which isn't an ancestor of `chapters/`)
    // is what surfaces an absolute `doc.name` and exercises the parent-prefix fallback.
    std::fs::write(dir.join("bluepencil.toml"), "project.files = [\"chapters/*.md\"]\n").unwrap();
    std::fs::write(chapters.join("one.md"), "One two three.\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "first"]);

    std::fs::write(chapters.join("one.md"), "One two three four five.\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "second"]);

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&other)
        .args(["--json", "progress", "--since", "HEAD~1"])
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total_before"], 3);
    assert_eq!(json["total_after"], 5);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn resolves_a_relative_path_containing_dot_dot() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);

    std::fs::write(dir.join("chapter.md"), "One two three.\n").unwrap();
    git(&dir, &["add", "chapter.md"]);
    git(&dir, &["commit", "-q", "-m", "first"]);

    std::fs::write(dir.join("chapter.md"), "One two three four five.\n").unwrap();
    git(&dir, &["add", "chapter.md"]);
    git(&dir, &["commit", "-q", "-m", "second"]);

    // `git show <rev>:<path>` does not normalize a `..` the way a shell would, so this must
    // resolve through the parent directory's own git-reported prefix rather than string
    // concatenation, or it silently reports the file as brand new (before == 0).
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&chapters)
        .args(["--json", "progress", "--since", "HEAD~1", "../chapter.md"])
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total_before"], 3);
    assert_eq!(json["total_after"], 5);

    std::fs::remove_dir_all(&dir).ok();
}
