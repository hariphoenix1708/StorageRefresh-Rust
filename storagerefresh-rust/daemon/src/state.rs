use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppState {
    pub last_run_timestamp: i64,
    pub last_run_result: String,
    pub run_count: u64,
    pub schema_version: u32,
    pub detected_environment: Option<crate::env_detect::Environment>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_run_timestamp: 0,
            last_run_result: "never_run".to_string(),
            run_count: 0,
            schema_version: 1,
            detected_environment: None,
        }
    }
}

impl AppState {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return state;
            }
        }
        AppState::default()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
