use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_dir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("bluepencil-init-test-{nanos}-{}-{n}", std::process::id()))
}

#[test]
fn genre_preset_matches_the_checked_in_example() {
    for (genre, example) in [
        ("novel", "examples/novel/bluepencil.toml"),
        ("essay", "examples/essay/bluepencil.toml"),
        ("screenplay", "examples/screenplay/bluepencil.toml"),
    ] {
        let dir = unique_dir();
        std::fs::create_dir_all(&dir).unwrap();

        let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
            .current_dir(&dir)
            .args(["init", "--genre", genre])
            .output()
            .expect("running bluepencil");
        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

        let written = std::fs::read_to_string(dir.join("bluepencil.toml")).unwrap();
        let expected =
            std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../").join(example)).unwrap();
        assert_eq!(written, expected, "init --genre {genre} drifted from {example}");

        std::fs::remove_dir_all(&dir).ok();
    }
}

#[test]
fn no_genre_writes_the_generic_template() {
    let dir = unique_dir();
    std::fs::create_dir_all(&dir).unwrap();

    let output =
        Command::new(env!("CARGO_BIN_EXE_bluepencil")).current_dir(&dir).arg("init").output().expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let written = std::fs::read_to_string(dir.join("bluepencil.toml")).unwrap();
    assert!(written.contains("Every setting is optional"), "expected the generic template, got:\n{written}");

    std::fs::remove_dir_all(&dir).ok();
}
