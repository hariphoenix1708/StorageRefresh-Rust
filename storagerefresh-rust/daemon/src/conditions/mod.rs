pub mod battery;
pub mod foreground_app;
pub mod idle;
pub mod screen;

pub use battery::check_battery;
pub use foreground_app::check_foreground_app;
pub use idle::check_idle;
pub use screen::check_screen;

use crate::config::Config;
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct ConditionsResult {
    pub battery_ok: bool,
    pub screen_ok: bool,
    pub idle_ok: bool,
    pub foreground_ok: bool,
    pub all_met: bool,
}

pub fn check_all_conditions(config: &Config, state: &AppState) -> ConditionsResult {
    let battery_ok = check_battery(&config.battery);
    let screen_ok = check_screen(config.screen.min_idle_minutes, state.screen_off_since);
    let idle_ok = check_idle(config.screen.min_idle_minutes, state.screen_off_since);
    let foreground_ok = if config.safety.require_foreground_check {
        check_foreground_app(&config.safety.foreground_denylist)
    } else {
        true
    };

    ConditionsResult {
        battery_ok,
        screen_ok,
        idle_ok,
        foreground_ok,
        all_met: battery_ok && screen_ok && idle_ok && foreground_ok,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_met_requires_every_condition() {
        let result = ConditionsResult {
            battery_ok: true,
            screen_ok: true,
            idle_ok: true,
            foreground_ok: false,
            all_met: false,
        };
        assert!(!result.all_met);
        let result = ConditionsResult {
            battery_ok: true,
            screen_ok: true,
            idle_ok: true,
            foreground_ok: true,
            all_met: true,
        };
        assert!(result.all_met);
    }
}
