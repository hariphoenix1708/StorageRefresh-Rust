use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

fn default_min_interval_hours() -> u64 {
    24
}

fn default_poll_interval_minutes() -> u64 {
    20
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ScheduleConfig {
    pub min_interval_hours: u64,
    pub poll_interval_minutes: u64,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            min_interval_hours: default_min_interval_hours(),
            poll_interval_minutes: default_poll_interval_minutes(),
        }
    }
}

fn default_charging_override_capacity() -> u8 {
    80
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct BatteryConfig {
    pub require_charging: bool,
    pub min_capacity_percent: u8,
    pub max_temperature_c: f32,
    /// Capacity at which TRIM is allowed even when not charging
    /// (handles devices whose charging state is unreliable).
    pub charging_override_capacity: u8,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            require_charging: true,
            min_capacity_percent: 30,
            max_temperature_c: 40.0,
            charging_override_capacity: default_charging_override_capacity(),
        }
    }
}

fn default_min_idle_minutes() -> u64 {
    10
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ScreenConfig {
    pub min_idle_minutes: u64,
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self {
            min_idle_minutes: default_min_idle_minutes(),
        }
    }
}

fn default_min_trim_len_mb() -> u64 {
    4
}

fn default_allowed_fs() -> Vec<String> {
    vec!["ext4".into(), "f2fs".into()]
}

fn default_mount_point() -> String {
    "/data".into()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct StorageConfig {
    pub min_trim_len_mb: u64,
    pub allowed_fs: Vec<String>,
    /// Mount point targeted by TRIM (e.g. "/data").
    pub mount_point: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            min_trim_len_mb: default_min_trim_len_mb(),
            allowed_fs: default_allowed_fs(),
            mount_point: default_mount_point(),
        }
    }
}

fn default_foreground_denylist() -> Vec<String> {
    vec![
        "game".into(),
        "camera".into(),
        "video".into(),
        "youtube".into(),
        "netflix".into(),
    ]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SafetyConfig {
    pub require_foreground_check: bool,
    pub dry_run: bool,
    /// Package-name substrings that block TRIM when resumed.
    pub foreground_denylist: Vec<String>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            require_foreground_check: true,
            dry_run: false,
            foreground_denylist: default_foreground_denylist(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
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

        config.clamp();

        Ok(config)
    }

    /// Enforce sane bounds on every tunable so a mis-typed config file can
    /// never cause a busy loop, a run on a nearly-dead battery, or an
    /// out-of-range temperature comparison.
    fn clamp(&mut self) {
        self.schedule.min_interval_hours = self.schedule.min_interval_hours.clamp(6, 168);
        self.schedule.poll_interval_minutes = self.schedule.poll_interval_minutes.clamp(1, 1440);
        self.battery.min_capacity_percent = self.battery.min_capacity_percent.clamp(1, 100);
        self.battery.charging_override_capacity =
            self.battery.charging_override_capacity.clamp(1, 100);
        self.battery.max_temperature_c = self.battery.max_temperature_c.clamp(0.0, 100.0);
        self.screen.min_idle_minutes = self.screen.min_idle_minutes.clamp(1, 10_080);
        self.storage.min_trim_len_mb = self.storage.min_trim_len_mb.clamp(0, 65_536);
        if self.storage.mount_point.is_empty() {
            self.storage.mount_point = default_mount_point();
        }
        if self.storage.allowed_fs.is_empty() {
            self.storage.allowed_fs = default_allowed_fs();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.schedule.min_interval_hours, 24);
        assert_eq!(config.schedule.poll_interval_minutes, 20);
        assert_eq!(config.battery.charging_override_capacity, 80);
        assert_eq!(config.storage.mount_point, "/data");
    }

    #[test]
    fn test_bounds_are_clamped() {
        let mut config = Config::default();
        config.schedule.min_interval_hours = 1;
        config.schedule.poll_interval_minutes = 0;
        config.battery.min_capacity_percent = 0;
        config.battery.charging_override_capacity = 200;
        config.battery.max_temperature_c = 500.0;
        config.screen.min_idle_minutes = 0;
        config.storage.min_trim_len_mb = u64::MAX;
        config.clamp();

        assert_eq!(config.schedule.min_interval_hours, 6);
        assert_eq!(config.schedule.poll_interval_minutes, 1);
        assert_eq!(config.battery.min_capacity_percent, 1);
        assert_eq!(config.battery.charging_override_capacity, 100);
        assert_eq!(config.battery.max_temperature_c, 100.0);
        assert_eq!(config.screen.min_idle_minutes, 1);
        assert_eq!(config.storage.min_trim_len_mb, 65_536);
    }

    #[test]
    fn test_partial_toml_uses_defaults_for_missing_fields() {
        // Older config files lack the newly added fields; they must still parse.
        let toml_str = r#"
            [schedule]
            min_interval_hours = 12
        "#;
        let config: Config = toml::from_str(toml_str).expect("partial toml must parse");
        assert_eq!(config.schedule.min_interval_hours, 12);
        assert_eq!(config.schedule.poll_interval_minutes, 20);
        assert_eq!(config.battery.charging_override_capacity, 80);
        assert_eq!(config.safety.foreground_denylist.len(), 5);
        assert_eq!(config.storage.mount_point, "/data");
    }

    #[test]
    fn test_empty_config_file_uses_defaults() {
        let dir = std::env::temp_dir();
        let path = dir.join("storagerefresh-empty-config-test.toml");
        std::fs::write(&path, "").unwrap();
        let config = Config::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(config.schedule.min_interval_hours, 24);
    }
}
