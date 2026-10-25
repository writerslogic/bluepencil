use std::io::Write;
use std::process::Command;

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[test]
fn count_reads_native_writing_files_through_cli() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("scene.fdx"), r#"<FinalDraft><Content><Paragraph Type="Action"><Text>She walked home.</Text></Paragraph></Content></FinalDraft>"#).unwrap();
    std::fs::write(
        dir.path().join("notes.tbx"),
        r#"<tinderbox><item><attribute name="Name">Scene</attribute><text>She walked home.</text></item></tinderbox>"#,
    )
    .unwrap();

    let zip_file = std::fs::File::create(dir.path().join("sheet.ulyz")).unwrap();
    let mut zip = ZipWriter::new(zip_file);
    zip.start_file("Sheet.ulysses/Contents.xml", SimpleFileOptions::default()).unwrap();
    zip.write_all(br#"<sheet><string><p>She walked home.</p></string></sheet>"#).unwrap();
    zip.finish().unwrap();

    let project = dir.path().join("novel.scriv");
    std::fs::create_dir_all(project.join("Files/Data/ONE")).unwrap();
    std::fs::write(project.join("novel.scrivx"), r#"<ScrivenerProject><Binder><BinderItem Type="DraftFolder" UUID="DRAFT"><Title>Draft</Title><Children><BinderItem Type="Text" UUID="ONE"><Title>Chapter</Title></BinderItem></Children></BinderItem></Binder></ScrivenerProject>"#).unwrap();
    std::fs::write(project.join("Files/Data/ONE/content.rtf"), br#"{\rtf1\ansi She walked home.}"#).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(dir.path())
        .args(["--json", "count", "scene.fdx", "notes.tbx", "sheet.ulyz", "novel.scriv"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["total"]["words"], 12);

    let globbed = Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .current_dir(dir.path())
        .args(["--json", "count", "**/*"])
        .output()
        .unwrap();
    assert!(globbed.status.success(), "{}", String::from_utf8_lossy(&globbed.stderr));
    let result: serde_json::Value = serde_json::from_slice(&globbed.stdout).unwrap();
    assert_eq!(result["total"]["words"], 12);
}
