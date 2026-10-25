use std::process::Command;

#[test]
fn report_points_to_revisable_passages_and_escapes_html() {
    let dir = tempfile::tempdir().unwrap();
    let name = "draft<script>.md";
    std::fs::write(dir.path().join(name), "The gravel was cold. The gravel was wet.\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(dir.path())
        .args(["--json", "report", name, "--html", "report.html"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let revision = result["revisions"].as_array().unwrap().iter().find(|item| item["rule"] == "echo").unwrap();
    assert_eq!(revision["file"], name);
    assert_eq!(revision["line"], 1);
    assert!(revision["excerpt"].as_str().unwrap().contains("gravel"));
    let html = std::fs::read_to_string(dir.path().join("report.html")).unwrap();
    assert!(html.contains("draft&lt;script&gt;.md"));
    assert!(!html.contains("draft<script>.md"));
}

#[test]
fn report_limits_repeated_rules_to_surface_varied_observations() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("draft.md"),
        "The gravel was cold. The gravel was wet. The gravel was dark. She looked at the road. She looked at the sky. She looked at the house. It was very quiet.\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(dir.path())
        .args(["--json", "report", "draft.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let revisions = result["revisions"].as_array().unwrap();
    assert!(revisions.iter().filter(|item| item["rule"] == "echo").count() <= 3);
    assert!(revisions.iter().any(|item| item["rule"] != "echo"));
}
