use serde::{Deserialize, Deserializer, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighscoreEntry {
    pub time_ms: u128,
    #[serde(default, deserialize_with = "deserialize_opt_string_from_any")]
    pub seed: Option<String>,
    pub clues: Option<usize>,
    pub date_utc: String,
    // If no seed was used, store the finished 81-char grid so it can be reloaded
    pub solution_sdk: Option<String>,
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

fn deserialize_opt_string_from_any<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize as _;
    use serde::de::Error as _;
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Number(n) => Ok(Some(n.to_string())),
        other => Err(D::Error::custom(format!("invalid seed type: {}", other))),
    }
}
