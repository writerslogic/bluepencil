use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn unique_dir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("bluepencil-docx-test-{nanos}-{}-{n}", std::process::id()))
}

const NS: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#;

fn write_docx(path: &std::path::Path, body: &str) {
    let document_xml =
        format!(r#"<?xml version="1.0" encoding="UTF-8"?><w:document {NS}><w:body>{body}</w:body></w:document>"#);
    let file = std::fs::File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(document_xml.as_bytes()).unwrap();
    zip.finish().unwrap();
}

#[test]
fn counts_words_in_a_docx_file() {
    let dir = unique_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("chapter.docx");
    write_docx(
        &path,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Chapter One</w:t></w:r></w:p>
           <w:p><w:r><w:t>One two three four five six seven eight nine ten.</w:t></w:r></w:p>"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "count", "chapter.docx"])
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // "Chapter One" is a Heading1 paragraph, so its two words are section structure, not prose.
    assert_eq!(json["total"]["words"], 10);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn explicit_as_docx_overrides_a_non_docx_extension() {
    let dir = unique_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("chapter.bin");
    write_docx(&path, r#"<w:p><w:r><w:t>One two three.</w:t></w:r></w:p>"#);

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(&dir)
        .args(["--json", "--as", "docx", "count", "chapter.bin"])
        .output()
        .expect("running bluepencil");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total"]["words"], 3);

    std::fs::remove_dir_all(&dir).ok();
}
