use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighscoreEntry {
    pub time_ms: u128,
    pub seed: Option<u64>,
    pub clues: Option<usize>,
    pub date_utc: String,
}

pub fn load<P: AsRef<Path>>(path: P) -> Vec<HighscoreEntry> {
    match fs::read_to_string(path) {
        Ok(s) => serde_json::from_str::<Vec<HighscoreEntry>>(&s).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save<P: AsRef<Path>>(path: P, list: &[HighscoreEntry]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(list).unwrap();
    fs::write(path, json)
}
