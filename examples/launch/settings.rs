//! Settings persisted between runs as a plain `key=value` file next to the
//! executable: no serialisation crate, readable with any editor.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

const FILE: &str = "poemulti.settings";

#[derive(Debug, Default, Clone)]
pub struct Settings(BTreeMap<String, String>);

impl Settings {
    pub fn path() -> PathBuf {
        crate::assets::data_dir().join(FILE)
    }

    /// Loads the file; a missing or unreadable file yields empty settings.
    pub fn load() -> Self {
        let mut map = BTreeMap::new();
        if let Ok(text) = std::fs::read_to_string(Self::path()) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        Self(map)
    }

    pub fn save(&self) -> std::io::Result<()> {
        let mut out = String::from("# PoEMulti demo settings, written on exit\n");
        for (k, v) in &self.0 {
            out.push_str(k);
            out.push('=');
            out.push_str(v);
            out.push('\n');
        }
        std::fs::write(Self::path(), out)
    }

    /// Parsed value of `key`, or `None` when absent or malformed.
    pub fn get<T: FromStr>(&self, key: &str) -> Option<T> {
        self.0.get(key).and_then(|v| v.parse().ok())
    }

    /// `get` with a fallback.
    pub fn get_or<T: FromStr>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }

    pub fn set(&mut self, key: &str, value: impl Display) {
        self.0.insert(key.to_string(), value.to_string());
    }
}
