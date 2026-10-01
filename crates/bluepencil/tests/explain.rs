//! `--explain` against a local stand-in for the Claude API. No network, no key.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::mpsc;

/// Serves one HTTP request with a canned streamed Messages API reply whose text is
/// `reply_text`, and hands back the raw request (headers and body).
fn one_shot_server(reply_text: &str) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    // Split the text across two deltas so the client has to concatenate.
    let mid = reply_text.len() / 2;
    let delta = |t: &str| {
        serde_json::json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": t}})
            .to_string()
    };
    let sse = format!(
        "event: message_start\ndata: {{\"type\":\"message_start\"}}\n\n\
         event: content_block_delta\ndata: {}\n\n\
         event: content_block_delta\ndata: {}\n\n\
         event: message_delta\ndata: {{\"type\":\"message_delta\",\"delta\":{{\"stop_reason\":\"end_turn\"}}}}\n\n\
         event: message_stop\ndata: {{\"type\":\"message_stop\"}}\n\n",
        delta(&reply_text[..mid]),
        delta(&reply_text[mid..]),
    );
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        let body_start;
        loop {
            let n = stream.read(&mut chunk).unwrap();
            buf.extend_from_slice(&chunk[..n]);
            if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                body_start = i + 4;
                break;
            }
        }
        let head = String::from_utf8_lossy(&buf[..body_start]).to_string();
        let len: usize = head
            .lines()
            .find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse().unwrap()))
            .unwrap();
        while buf.len() < body_start + len {
            let n = stream.read(&mut chunk).unwrap();
            buf.extend_from_slice(&chunk[..n]);
        }
        tx.send(head + &String::from_utf8_lossy(&buf[body_start..body_start + len])).unwrap();
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            sse.len(),
            sse
        );
    });
    (format!("http://{addr}"), rx)
}

fn workspace(base_url: &str) -> tempdir::Dir {
    let dir = tempdir::Dir::new();
    std::fs::write(
        dir.path.join("bluepencil.toml"),
        format!("[model]\nbase_url = \"{base_url}\"\napi_key_env = \"BLUEPENCIL_TEST_KEY\"\nmax_findings = 1\n"),
    )
    .unwrap();
    std::fs::write(
        dir.path.join("ch.md"),
        "He was very tired. She was really quite done with it.\n\nHer grey eyes shone.\n\nMara's brown eyes caught the light.\n",
    )
    .unwrap();
    dir
}

fn bluepencil(dir: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bluepencil"));
    cmd.current_dir(dir).stdin(std::process::Stdio::null());
    cmd
}

#[test]
fn explain_sends_findings_with_context_and_prints_the_note() {
    let reply = r#"{"notes": [{"index": 0, "verdict": "fix", "note": "Cut it: \"He was tired.\""}]}"#;
    let (url, rx) = one_shot_server(reply);
    let dir = workspace(&url);
    let out =
        bluepencil(&dir.path).args(["hedges", "ch.md", "--explain"]).env("BLUEPENCIL_TEST_KEY", "k").output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let request = rx.recv().unwrap();
    assert!(request.contains("x-api-key: k"), "api key header missing:\n{request}");
    assert!(request.contains("anthropic-version: 2023-06-01"));
    let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["output_config"]["format"]["type"], "json_schema");
    let user = body["messages"][0]["content"].as_str().unwrap();
    assert!(user.contains("Annotate these 1 findings"), "max_findings = 1 should cap the request:\n{user}");
    assert!(user.contains("\"rule\": \"hedge\""));
    assert!(user.contains("He was very tired."), "context sentence missing:\n{user}");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("→ fix: Cut it"), "note not printed:\n{stdout}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("explaining the first 1 of"), "cap not reported:\n{stderr}");
}

#[test]
fn explain_json_attaches_notes_to_findings() {
    let reply = r#"{"notes": [{"index": 0, "verdict": "keep", "note": "Idiomatic here."}]}"#;
    let (url, _rx) = one_shot_server(reply);
    let dir = workspace(&url);
    let out = bluepencil(&dir.path)
        .args(["hedges", "ch.md", "--explain", "--json"])
        .env("BLUEPENCIL_TEST_KEY", "k")
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let findings = json["findings"].as_array().unwrap();
    assert_eq!(findings[0]["note"]["verdict"], "keep");
    assert_eq!(findings[0]["note"]["note"], "Idiomatic here.");
    assert!(findings[1].get("note").is_none(), "unexplained findings carry no note key");
}

#[test]
fn facts_pins_quotes_to_real_positions_and_prints_contradictions() {
    let reply = serde_json::json!({
        "facts": [
            {"entity": "Mara", "attribute": "eye color", "value": "grey", "file": "ch.md", "line": 3, "quote": "grey eyes"},
            {"entity": "Mara", "attribute": "eye color", "value": "brown", "file": "ch.md", "line": 5, "quote": "brown eyes"},
            {"entity": "Mara", "attribute": "hair", "value": "red", "file": "ch.md", "line": 5, "quote": "red hair"}
        ],
        "contradictions": [
            {"entity": "Mara", "attribute": "eye color", "facts": [0, 1], "note": "Grey, then brown."}
        ]
    })
    .to_string();
    let (url, rx) = one_shot_server(&reply);
    let dir = workspace(&url);
    let out =
        bluepencil(&dir.path).args(["facts", "ch.md", "--ledger"]).env("BLUEPENCIL_TEST_KEY", "k").output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let request = rx.recv().unwrap();
    let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["stream"], true);
    let user = body["messages"][0]["content"].as_str().unwrap();
    assert!(user.contains("ch.md:3| Her grey eyes shone."), "numbered lines missing:\n{user}");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Mara / eye color"), "{stdout}");
    assert!(stdout.contains("ch.md:3:5  grey"), "quote should be pinned to its real column:\n{stdout}");
    assert!(stdout.contains("ch.md:5:8  brown"), "{stdout}");
    assert!(
        stdout.contains("ch.md:5  Mara / hair: red  (quote not found verbatim)"),
        "unfound quote falls back to the cited line:\n{stdout}"
    );
    assert!(stdout.contains("→ Grey, then brown."));
    assert!(stdout.contains("1 contradictions across 3 facts, 1 quotes not found verbatim"), "{stdout}");
}

#[test]
fn facts_json_marks_verification() {
    let reply = serde_json::json!({
        "facts": [{"entity": "Mara", "attribute": "eye color", "value": "grey", "file": "ch.md", "line": 3, "quote": "grey eyes"}],
        "contradictions": []
    })
    .to_string();
    let (url, _rx) = one_shot_server(&reply);
    let dir = workspace(&url);
    let out =
        bluepencil(&dir.path).args(["facts", "ch.md", "--json"]).env("BLUEPENCIL_TEST_KEY", "k").output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["facts"][0]["verified"], true);
    assert_eq!(json["facts"][0]["location"], "ch.md:3:5");
    assert_eq!(json["contradictions"].as_array().unwrap().len(), 0);
}

#[test]
fn voice_prints_profiles_and_pinned_off_voice_lines() {
    let reply = serde_json::json!({
        "characters": [
            {"name": "Mara", "lines": 12, "profile": "Short declaratives, no contractions."},
            {"name": "Tom", "lines": 30, "profile": "Rambling, slangy."}
        ],
        "findings": [
            {"character": "Mara", "file": "ch.md", "line": 5, "quote": "caught the light", "note": "Sounds like the narrator."}
        ]
    })
    .to_string();
    let (url, _rx) = one_shot_server(&reply);
    let dir = workspace(&url);
    let out = bluepencil(&dir.path).args(["voice", "ch.md"]).env("BLUEPENCIL_TEST_KEY", "k").output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let tom = stdout.find("Tom (30 lines)").expect("Tom listed");
    let mara = stdout.find("Mara (12 lines)").expect("Mara listed");
    assert!(tom < mara, "characters sort by line count:\n{stdout}");
    assert!(stdout.contains("ch.md:5:19  Mara"), "{stdout}");
    assert!(stdout.contains("→ Sounds like the narrator."));
}

#[test]
fn scenes_sends_the_pacing_profile_and_prints_verdicts() {
    let reply = serde_json::json!({
        "scenes": [
            {"title": "Tired", "file": "ch.md", "line": 1, "opening": "He was very tired", "goal": "rest", "conflict": "none", "change": "none", "verdict": "cut", "note": "Nothing turns."},
            {"title": "Eyes", "file": "ch.md", "line": 3, "opening": "Her grey eyes", "goal": "g", "conflict": "c", "change": "d", "verdict": "keep", "note": "Fine."}
        ]
    })
    .to_string();
    let (url, rx) = one_shot_server(&reply);
    let dir = workspace(&url);
    let out = bluepencil(&dir.path).args(["scenes", "ch.md"]).env("BLUEPENCIL_TEST_KEY", "k").output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let request = rx.recv().unwrap();
    let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    let user = body["messages"][0]["content"].as_str().unwrap();
    assert!(user.contains("Measured pacing profile"), "arc profile should precede the text:\n{user}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ch.md:1:1  Tired"), "{stdout}");
    assert!(stdout.contains("→ cut: Nothing turns."));
    assert!(stdout.contains("2 scenes, 1 to look at"), "{stdout}");
}

#[test]
fn config_shows_the_effective_configuration_and_its_source() {
    let dir = workspace("http://127.0.0.1:9");
    let out = bluepencil(&dir.path).args(["config"]).output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("# effective configuration from "), "{stdout}");
    assert!(stdout.contains("bluepencil.toml"));
    assert!(stdout.contains("max_findings = 1"), "file value should win:\n{stdout}");
    assert!(stdout.contains("window = 50"), "defaults should be filled in:\n{stdout}");

    let out = bluepencil(&dir.path).args(["config", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["path"].as_str().unwrap().ends_with("bluepencil.toml"));
    assert_eq!(json["config"]["model"]["max_findings"], 1);
}

#[test]
fn explain_without_a_key_fails_before_any_request() {
    let dir = workspace("http://127.0.0.1:9");
    let out = bluepencil(&dir.path)
        .args(["hedges", "ch.md", "--explain"])
        .env_remove("BLUEPENCIL_TEST_KEY")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("BLUEPENCIL_TEST_KEY"), "should name the variable to set:\n{stderr}");
}

#[test]
fn explain_is_rejected_on_commands_that_do_not_support_it() {
    let dir = workspace("http://127.0.0.1:9");
    let out =
        bluepencil(&dir.path).args(["count", "ch.md", "--explain"]).env("BLUEPENCIL_TEST_KEY", "k").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not supported"));
}

mod tempdir {
    use std::path::PathBuf;

    pub struct Dir {
        pub path: PathBuf,
    }

    impl Dir {
        pub fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "bluepencil-explain-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
