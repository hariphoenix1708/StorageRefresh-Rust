pub mod battery;
pub mod foreground_app;
pub mod idle;
pub mod screen;

pub use battery::check_battery;
pub use foreground_app::check_foreground_app;
pub use idle::check_idle;
pub use screen::check_screen;

#[derive(Debug, serde::Serialize)]
pub struct ConditionsResult {
    pub battery_ok: bool,
    pub screen_ok: bool,
    pub idle_ok: bool,
    pub foreground_ok: bool,
    pub all_met: bool,
}

pub fn check_all_conditions(
    min_capacity: u8,
    require_charging: bool,
    max_temp_c: f32,
    min_idle_minutes: u64,
    require_foreground_check: bool,
) -> ConditionsResult {
    let battery_ok = check_battery(min_capacity, require_charging, max_temp_c);
    let screen_ok = check_screen(min_idle_minutes);
    let idle_ok = check_idle(min_idle_minutes);
    let foreground_ok = if require_foreground_check {
        check_foreground_app()
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
