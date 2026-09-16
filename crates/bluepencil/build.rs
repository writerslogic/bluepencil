use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| format!(" ({})", s.trim()))
        .unwrap_or_default();
    println!("cargo:rustc-env=BLUEPENCIL_GIT_HASH={hash}");
}
