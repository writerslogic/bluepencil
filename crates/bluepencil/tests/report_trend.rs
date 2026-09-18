use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_dir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("bluepencil-report-trend-test-{nanos}-{}-{n}", std::process::id()))
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .status()
        .expect("running git");
    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

#[test]
fn trend_since_without_html_is_rejected() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);
    std::fs::write(chapters.join("one.md"), "One two three.\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "first"]);

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["report", "--trend-since", "HEAD", "chapters/one.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--html"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn trend_chart_reflects_word_count_growth_across_commits() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);

    let mut body = String::from("One two three four five.");
    for i in 0..6 {
        body.push_str(&format!(" word{i}"));
        std::fs::write(chapters.join("one.md"), format!("{body}\n")).unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", &format!("chunk {i}")]);
    }
    let first =
        String::from_utf8(Command::new("git").current_dir(&dir).args(["rev-list", "--all"]).output().unwrap().stdout)
            .unwrap()
            .lines()
            .next_back()
            .unwrap()
            .to_string();

    let html_path = dir.join("out.html");
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args([
            "report",
            "--html",
            html_path.to_str().unwrap(),
            "--trend-since",
            &first,
            "--trend-points",
            "4",
            "chapters/one.md",
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let html = std::fs::read_to_string(&html_path).unwrap();
    assert!(html.contains("<section><h2>Trend</h2>"), "expected a trend section:\n{html}");
    let polylines = html.matches("<polyline").count();
    assert_eq!(polylines, 4, "expected one word-count line and three flag lines, got {polylines}");
    assert!(!html.contains("{{trend}}"), "template placeholder leaked into output");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn no_trend_since_omits_the_trend_section() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@example.com"]);
    git(&dir, &["config", "user.name", "Test"]);
    std::fs::write(chapters.join("one.md"), "One two three.\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "first"]);

    let html_path = dir.join("out.html");
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["report", "--html", html_path.to_str().unwrap(), "chapters/one.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let html = std::fs::read_to_string(&html_path).unwrap();
    assert!(!html.contains("<h2>Trend</h2>"));
    assert!(!html.contains("{{trend}}"));

    std::fs::remove_dir_all(&dir).ok();
}
