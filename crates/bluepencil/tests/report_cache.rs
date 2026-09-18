use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_dir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("bluepencil-report-cache-test-{nanos}-{}-{n}", std::process::id()))
}

fn run_report(dir: &std::path::Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(dir)
        .args(["report", "--json", "one.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cached_rerun_on_unchanged_file_reports_identical_output() {
    let dir = unique_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("one.md"), "One two three four five. Six seven eight nine ten.\n").unwrap();

    let first = run_report(&dir);
    assert!(dir.join(".bluepencil-cache").is_dir(), "expected the cache dir to be created");
    let second = run_report(&dir);

    assert_eq!(first, second, "a cached rerun on unchanged content must match the first run exactly");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_cache_entry_from_a_single_file_run_does_not_leak_its_name_into_a_later_multi_file_run() {
    let dir = unique_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("solo.md"), "One two three four five six seven.\n").unwrap();
    std::fs::write(dir.join("other.md"), "Eight nine ten.\n").unwrap();

    // A single-file report caches solo.md's Metrics under the label "all files" (that call's
    // `total`, since docs.len() == 1). The cache key is content + config only, not that label,
    // so a later multi-file run must still show solo.md under its own name in the per-file
    // breakdown rather than the stale "all files" label leaking through the cache hit.
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["report", "--json", "solo.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["report", "--json", "solo.md", "other.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let names: Vec<&str> = json["files"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["solo.md", "other.md"], "cached entry leaked its old label: {json}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn changed_content_invalidates_the_cache() {
    let dir = unique_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("one.md"), "One two three four five.\n").unwrap();

    let first = run_report(&dir);

    std::fs::write(dir.join("one.md"), "One two three four five six seven eight nine ten eleven twelve.\n").unwrap();
    let second = run_report(&dir);

    assert_ne!(first, second, "changed content must not be served from a stale cache entry");

    std::fs::remove_dir_all(&dir).ok();
}
