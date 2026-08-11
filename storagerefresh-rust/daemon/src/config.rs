use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleConfig {
    pub min_interval_hours: u64,
    pub poll_interval_minutes: u64,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            min_interval_hours: 24,
            poll_interval_minutes: 20,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatteryConfig {
    pub require_charging: bool,
    pub min_capacity_percent: u8,
    pub max_temperature_c: f32,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            require_charging: true,
            min_capacity_percent: 30,
            max_temperature_c: 40.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScreenConfig {
    pub min_idle_minutes: u64,
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self {
            min_idle_minutes: 10,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub min_trim_len_mb: u64,
    pub allowed_fs: Vec<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            min_trim_len_mb: 4,
            allowed_fs: vec!["ext4".into(), "f2fs".into()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetyConfig {
    pub require_foreground_check: bool,
    pub dry_run: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            require_foreground_check: true,
            dry_run: false,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    pub schedule: ScheduleConfig,
    pub battery: BatteryConfig,
    pub screen: ScreenConfig,
    pub storage: StorageConfig,
    pub safety: SafetyConfig,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;

        // Bounds enforcement
        if config.schedule.min_interval_hours < 6 {
            config.schedule.min_interval_hours = 6;
        } else if config.schedule.min_interval_hours > 168 { // 7 days
            config.schedule.min_interval_hours = 168;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.schedule.min_interval_hours, 24);
    }
}
