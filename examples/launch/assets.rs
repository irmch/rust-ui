//! Where the demo keeps its files: assets are looked up next to the
//! executable first (an installed build), then up the `target/` tree and the
//! source checkout (a `cargo run`); settings and the saved log go next to the
//! executable.

use std::path::PathBuf;

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// First existing `assets` directory among the usual places.
pub fn asset_dir() -> PathBuf {
    let exe = exe_dir();
    let mut candidates = vec![exe.join("assets")];
    // target/{debug,release}/examples/launch.exe -> repo root is three up
    let mut up = exe.clone();
    for _ in 0..4 {
        if let Some(p) = up.parent() {
            up = p.to_path_buf();
            candidates.push(up.join("assets"));
        }
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
    candidates.push(PathBuf::from("assets"));
    candidates
        .into_iter()
        .find(|p| p.is_dir())
        .unwrap_or_else(|| PathBuf::from("assets"))
}

/// Directory for settings and logs: next to the executable.
pub fn data_dir() -> PathBuf {
    exe_dir()
}
