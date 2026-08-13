use std::fs;
use std::path::Path;

use crate::config::BatteryConfig;

/// Pure decision logic, separated from sysfs reads so it can be unit tested.
/// Returns true when the battery is safe for maintenance.
pub fn battery_pass(capacity: u8, is_charging: bool, temp_c: f32, config: &BatteryConfig) -> bool {
    if capacity < config.min_capacity_percent {
        return false;
    }

    if config.require_charging && !is_charging && capacity < config.charging_override_capacity {
        // Bias toward skipping: if charging is required but the status is
        // unreliable, still allow when capacity is high enough.
        return false;
    }

    if temp_c > config.max_temperature_c {
        return false;
    }

    true
}

pub fn check_battery(config: &BatteryConfig) -> bool {
    let capacity_str = read_sysfs("/sys/class/power_supply/battery/capacity").unwrap_or_default();
    let status_str = read_sysfs("/sys/class/power_supply/battery/status").unwrap_or_default();
    let temp_str = read_sysfs("/sys/class/power_supply/battery/temp").unwrap_or_default();

    // Fails safe: unreadable/unparseable values push toward skipping.
    let capacity: u8 = capacity_str.trim().parse().unwrap_or(0);
    let temp_tenths: i32 = temp_str.trim().parse().unwrap_or(9999);
    let temp_c = temp_tenths as f32 / 10.0;

    let is_charging = status_str.trim() == "Charging" || status_str.trim() == "Full";

    battery_pass(capacity, is_charging, temp_c, config)
}

fn read_sysfs<P: AsRef<Path>>(path: P) -> Option<String> {
    fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> BatteryConfig {
        BatteryConfig {
            require_charging: true,
            min_capacity_percent: 30,
            max_temperature_c: 40.0,
            charging_override_capacity: 80,
        }
    }

    #[test]
    fn test_low_capacity_blocks() {
        let cfg = test_config();
        assert!(!battery_pass(20, true, 25.0, &cfg));
    }

    #[test]
    fn test_charging_low_capacity_allowed() {
        let cfg = test_config();
        assert!(battery_pass(50, true, 25.0, &cfg));
    }

    #[test]
    fn test_not_charging_needs_high_capacity() {
        let cfg = test_config();
        assert!(!battery_pass(50, false, 25.0, &cfg));
        assert!(battery_pass(85, false, 25.0, &cfg));
    }

    #[test]
    fn test_overheating_blocks() {
        let cfg = test_config();
        assert!(!battery_pass(85, true, 41.0, &cfg));
        assert!(battery_pass(85, true, 39.9, &cfg));
    }

    #[test]
    fn test_require_charging_disabled() {
        let mut cfg = test_config();
        cfg.require_charging = false;
        assert!(battery_pass(50, false, 25.0, &cfg));
    }
}
