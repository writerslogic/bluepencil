//! Every versioned artifact in the repository carries the workspace version.
//! `RELEASING.md` lists the files to bump; this test fails CI when one is left behind.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(root().join(path)).unwrap_or_else(|err| panic!("{path}: {err}"))
}

fn json_version(path: &str, pointer: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(&read(path)).unwrap_or_else(|err| panic!("{path}: {err}"));
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("{path}: no string at {pointer}"))
        .to_owned()
}

#[test]
fn packages_share_the_workspace_version() {
    let expected = env!("CARGO_PKG_VERSION");
    let manifest: toml::Value = toml::from_str(&read("Cargo.toml")).expect("Cargo.toml parses");
    let mut found = vec![
        (
            "Cargo.toml workspace.package.version".to_owned(),
            manifest["workspace"]["package"]["version"].as_str().unwrap_or_default().to_owned(),
        ),
        (
            "Cargo.toml workspace.dependencies.bluepencil-core.version".to_owned(),
            manifest["workspace"]["dependencies"]["bluepencil-core"]["version"].as_str().unwrap_or_default().to_owned(),
        ),
    ];
    for (path, pointer) in [
        ("crates/bluepencil-node/package.json", "/version"),
        ("crates/bluepencil-node/package-lock.json", "/version"),
        ("crates/bluepencil-node/package-lock.json", "/packages//version"),
        ("editors/vscode/package.json", "/version"),
        ("editors/vscode/package-lock.json", "/version"),
        ("editors/vscode/package-lock.json", "/packages//version"),
    ] {
        found.push((format!("{path} {pointer}"), json_version(path, pointer)));
    }

    // `npm run build` regenerates index.js and bakes the version every per-platform
    // binding package must carry; a stale file publishes a package that rejects its
    // own bindings.
    let index = read("crates/bluepencil-node/index.js");
    let marker = "version mismatch, expected ";
    let baked: Vec<&str> =
        index.match_indices(marker).filter_map(|(at, _)| index[at + marker.len()..].split(' ').next()).collect();
    assert!(
        !baked.is_empty(),
        "crates/bluepencil-node/index.js has no binding version checks; run `npm run build` there"
    );
    for version in baked {
        found.push(("crates/bluepencil-node/index.js".to_owned(), version.to_owned()));
    }

    let stale: Vec<&(String, String)> = found.iter().filter(|(_, version)| version != expected).collect();
    assert!(stale.is_empty(), "workspace version is {expected}; out of sync: {stale:#?}");
}
