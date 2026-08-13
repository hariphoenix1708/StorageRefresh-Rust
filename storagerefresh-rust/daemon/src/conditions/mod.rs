pub mod battery;
pub mod foreground_app;
pub mod idle;
pub mod screen;

pub use battery::check_battery;
pub use foreground_app::check_foreground_app;
pub use idle::check_idle;

use crate::config::Config;
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct ConditionsResult {
    pub battery_ok: bool,
    pub screen_ok: bool,
    pub idle_ok: bool,
    pub foreground_ok: bool,
    pub all_met: bool,
    /// Human-readable explanation of why maintenance is (not) allowed.
    pub reason: String,
}

pub fn check_all_conditions(config: &Config, state: &AppState) -> ConditionsResult {
    let (battery_ok, battery_reason) = check_battery(&config.battery);
    let screen_state = screen::screen_state();
    let screen_off = screen_state == Some(false);
    let screen_ok = screen::screen_pass(
        screen_off,
        config.screen.min_idle_minutes,
        state.screen_off_since,
    );
    let idle_ok = check_idle(config.screen.min_idle_minutes, state.screen_off_since);
    let foreground_ok = if config.safety.require_foreground_check {
        check_foreground_app(&config.safety.foreground_denylist)
    } else {
        true
    };

    let all_met = battery_ok && screen_ok && idle_ok && foreground_ok;

    let reason = if all_met {
        "All conditions met".to_string()
    } else {
        let mut parts: Vec<String> = Vec::new();
        if !battery_ok {
            parts.push(battery_reason);
        }
        if !screen_ok {
            parts.push(match screen_state {
                Some(true) => "screen is on".to_string(),
                Some(false) => {
                    let mins = screen::screen_off_elapsed(state.screen_off_since).unwrap_or(0) / 60;
                    format!(
                        "screen off only ~{} min (need {} min)",
                        mins, config.screen.min_idle_minutes
                    )
                }
                None => "screen state unknown".to_string(),
            });
        }
        if !idle_ok {
            parts.push(format!(
                "idle duration < {} min",
                config.screen.min_idle_minutes
            ));
        }
        if !foreground_ok {
            parts.push("foreground app in denylist".to_string());
        }
        parts.join("; ")
    };

    ConditionsResult {
        battery_ok,
        screen_ok,
        idle_ok,
        foreground_ok,
        all_met,
        reason,
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
            reason: String::new(),
        };
        assert!(!result.all_met);
        let result = ConditionsResult {
            battery_ok: true,
            screen_ok: true,
            idle_ok: true,
            foreground_ok: true,
            all_met: true,
            reason: String::new(),
        };
        assert!(result.all_met);
    }
}
