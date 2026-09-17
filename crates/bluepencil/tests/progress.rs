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
