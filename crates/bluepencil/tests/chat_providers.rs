use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::mpsc;

fn server() -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = Vec::new();
        let mut chunk = [0u8; 4096];
        let header_end = loop {
            let n = stream.read(&mut chunk).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&chunk[..n]);
            if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let header = String::from_utf8_lossy(&bytes[..header_end]);
        let len: usize = header
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase().strip_prefix("content-length:").map(|value| value.trim().parse().unwrap())
            })
            .unwrap();
        while bytes.len() < header_end + len {
            let n = stream.read(&mut chunk).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&chunk[..n]);
        }
        tx.send(String::from_utf8_lossy(&bytes[..header_end + len]).into_owned()).unwrap();
        let note = r#"{"notes":[{"index":0,"verdict":"keep","note":"The hesitation suits the scene."}]}"#;
        let reply = serde_json::json!({"choices":[{"finish_reason":"stop","message":{"content":note}}]}).to_string();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            reply.len(),
            reply
        )
        .unwrap();
    });
    (format!("http://{addr}/v1"), rx)
}

#[test]
fn openai_provider_sends_schema_and_reads_explanation() {
    let (url, rx) = server();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("bluepencil.toml"), format!("[model]\nprovider = \"openai\"\nmodel = \"test-model\"\nbase_url = \"{url}\"\napi_key_env = \"BLUEPENCIL_CHAT_TEST_KEY\"\nmax_findings = 1\n")).unwrap();
    std::fs::write(dir.path().join("chapter.md"), "She was very tired.").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(dir.path())
        .args(["hedges", "chapter.md", "--explain", "--json"])
        .env("BLUEPENCIL_CHAT_TEST_KEY", "test-secret")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let request = rx.recv().unwrap();
    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(request.to_ascii_lowercase().contains("authorization: bearer test-secret"));
    let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["response_format"]["type"], "json_schema");
    assert_eq!(body["response_format"]["json_schema"]["strict"], true);
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["findings"][0]["note"]["verdict"], "keep");
}

#[test]
fn openrouter_and_compatible_providers_use_chat_completions() {
    for provider in ["openrouter", "compatible"] {
        let (url, rx) = server();
        let dir = tempfile::tempdir().unwrap();
        let key_setting = if provider == "openrouter" { "api_key_env = \"BLUEPENCIL_CHAT_TEST_KEY\"\n" } else { "" };
        std::fs::write(
            dir.path().join("bluepencil.toml"),
            format!("[model]\nprovider = \"{provider}\"\nmodel = \"test-model\"\nbase_url = \"{url}\"\n{key_setting}max_findings = 1\n"),
        )
        .unwrap();
        std::fs::write(dir.path().join("chapter.md"), "She was very tired.").unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
            .current_dir(dir.path())
            .args(["hedges", "chapter.md", "--explain", "--json"])
            .env("BLUEPENCIL_CHAT_TEST_KEY", "test-secret")
            .output()
            .unwrap();
        assert!(output.status.success(), "{provider}: {}", String::from_utf8_lossy(&output.stderr));
        let request = rx.recv().unwrap();
        assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
        assert_eq!(
            request.to_ascii_lowercase().contains("authorization: bearer test-secret"),
            provider == "openrouter"
        );
        let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert!(body.get("max_tokens").is_some());
        assert_eq!(body["response_format"]["type"], "json_schema");
    }
}
