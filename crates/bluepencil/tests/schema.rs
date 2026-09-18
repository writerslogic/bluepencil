use std::process::Command;

#[test]
fn count_schema_is_valid_json_with_the_expected_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .args(["schema", "count"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let schema: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(schema["title"], "CountOutput");
    assert!(schema["properties"]["files"].is_object());
    assert!(schema["properties"]["total"].is_object());
    assert!(schema["$defs"]["Counts"]["properties"]["words"].is_object());
}

#[test]
fn unmigrated_command_reports_no_schema_rather_than_a_wrong_one() {
    // `split` has no --json output at all (it's a file-manipulation command), so it's the
    // one genuinely unmigrated case left once every analysis command has a schema.
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .args(["schema", "split"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no schema"));
}

#[test]
fn every_migrated_command_prints_a_valid_schema() {
    for command in [
        "count",
        "adverbs",
        "cliches",
        "filter",
        "hedges",
        "passive",
        "tics",
        "check",
        "continuity",
        "freq",
        "progress",
        "echoes",
        "repeats",
        "wdiff",
        "overused",
        "dialogue",
        "outline",
        "rhythm",
        "starters",
        "readability",
        "report",
        "unique",
        "histogram",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
            .args(["schema", command])
            .stdin(std::process::Stdio::null())
            .output()
            .expect("running bluepencil");
        assert!(output.status.success(), "`schema {command}` failed: {}", String::from_utf8_lossy(&output.stderr));
        let schema: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|e| panic!("`schema {command}` did not print valid JSON: {e}"));
        assert!(schema["type"].is_string(), "`schema {command}` missing a top-level `type`: {schema}");
    }
}

#[test]
fn count_json_output_matches_the_schema_field_for_field() {
    let dir = std::env::temp_dir().join(format!("bluepencil-schema-count-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.md"), "One two three.\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "count", "a.md"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let counts = &json["files"][0]["counts"];
    for field in [
        "words",
        "characters",
        "characters_no_spaces",
        "sentences",
        "paragraphs",
        "reading_minutes",
        "speaking_minutes",
    ] {
        assert!(counts[field].is_number(), "missing field `{field}` in count output: {json}");
    }

    std::fs::remove_dir_all(&dir).ok();
}
