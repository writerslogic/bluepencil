use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_dir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("bluepencil-continuity-test-{nanos}-{}-{n}", std::process::id()))
}

#[test]
fn flags_a_spelling_drift_across_chapters() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    std::fs::write(chapters.join("one.md"), "I found Sara in the garden. Sara was reading nearby.\n").unwrap();
    std::fs::write(chapters.join("twenty.md"), "By now I knew Sarah well. Sarah had changed since then.\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "continuity", "chapters/one.md", "chapters/twenty.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let clusters = json["clusters"].as_array().unwrap();
    assert_eq!(clusters.len(), 1, "expected one cluster: {json}");
    let spellings: Vec<&str> =
        clusters[0]["variants"].as_array().unwrap().iter().map(|v| v["spelling"].as_str().unwrap()).collect();
    assert!(spellings.contains(&"sara"), "{spellings:?}");
    assert!(spellings.contains(&"sarah"), "{spellings:?}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn ignore_list_in_config_suppresses_a_cluster() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    std::fs::write(chapters.join("one.md"), "I found Sara in the garden. Sara was reading nearby.\n").unwrap();
    std::fs::write(chapters.join("twenty.md"), "By now I knew Sarah well. Sarah had changed since then.\n").unwrap();
    std::fs::write(dir.join("bluepencil.toml"), "[continuity]\nignore = [\"Sara\"]\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "continuity", "chapters/one.md", "chapters/twenty.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["clusters"].as_array().unwrap().is_empty(), "{json}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn distinct_names_are_not_flagged() {
    let dir = unique_dir();
    let chapters = dir.join("chapters");
    std::fs::create_dir_all(&chapters).unwrap();
    std::fs::write(
        chapters.join("one.md"),
        "I met Elaine and Marcus at the docks. I saw Elaine leave. I saw Marcus stay behind.\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "continuity", "chapters/one.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["clusters"].as_array().unwrap().is_empty(), "{json}");

    std::fs::remove_dir_all(&dir).ok();
}
