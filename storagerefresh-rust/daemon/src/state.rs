use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppState {
    pub last_run_timestamp: i64,
    pub last_run_result: String,
    pub run_count: u64,
    pub schema_version: u32,
    pub last_trimmed_bytes: u64,
    /// Unix timestamp of when the screen was first observed off, if any.
    pub screen_off_since: Option<i64>,
    pub detected_environment: Option<crate::env_detect::Environment>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_run_timestamp: 0,
            last_run_result: "never_run".to_string(),
            run_count: 0,
            schema_version: 2,
            last_trimmed_bytes: 0,
            screen_off_since: None,
            detected_environment: None,
        }
    }
}

impl AppState {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(mut state) = serde_json::from_str::<AppState>(&content) {
                // Keep the schema version current.
                state.schema_version = 2;
                return state;
            }
        }
        AppState::default()
    }

    /// Write state atomically (temp file + rename) so a crash mid-write can
    /// never leave a truncated/corrupt state file behind.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp = path.with_extension("json.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("storagerefresh-state-test.json");
        let state = AppState {
            last_run_timestamp: 1_700_000_000,
            run_count: 3,
            last_trimmed_bytes: 4096,
            screen_off_since: Some(1_699_999_000),
            ..AppState::default()
        };
        state.save(&path).unwrap();

        let loaded = AppState::load(&path);
        std::fs::remove_file(&path).ok();
        assert_eq!(loaded.last_run_timestamp, 1_700_000_000);
        assert_eq!(loaded.run_count, 3);
        assert_eq!(loaded.last_trimmed_bytes, 4096);
        assert_eq!(loaded.screen_off_since, Some(1_699_999_000));
        assert_eq!(loaded.schema_version, 2);
    }

    #[test]
    fn test_corrupt_state_falls_back_to_default() {
        let dir = std::env::temp_dir();
        let path = dir.join("storagerefresh-corrupt-state-test.json");
        fs::write(&path, "{not valid json").unwrap();
        let state = AppState::load(&path);
        std::fs::remove_file(&path).ok();
        assert_eq!(state.last_run_result, "never_run");
        assert_eq!(state.run_count, 0);
    }

    #[test]
    fn test_no_temp_file_left_behind() {
        let dir = std::env::temp_dir();
        let path = dir.join("storagerefresh-atomic-test.json");
        AppState::default().save(&path).unwrap();
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::remove_file(&path).ok();
    }
}
