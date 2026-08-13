use std::process::Command;

use chrono::Utc;

/// Returns `Some(true)` when the screen is on, `Some(false)` when it is off,
/// and `None` when the state could not be determined (fail-safe: skip).
pub fn screen_state() -> Option<bool> {
    let output = Command::new("dumpsys").arg("power").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);

    let screen_on = stdout.contains("mScreenOn=true")
        || stdout.contains("Display Power: state=ON")
        || stdout.contains("mWakefulness=Awake");

    Some(screen_on)
}

/// Seconds elapsed since the screen turned off, if known.
pub fn screen_off_elapsed(screen_off_since: Option<i64>) -> Option<u64> {
    screen_off_since.map(|since| Utc::now().timestamp().saturating_sub(since).max(0) as u64)
}

/// Pure decision logic: TRIM is only allowed while the screen is off AND has
/// been off continuously for at least `min_idle_minutes`.
pub fn screen_pass(
    screen_is_off: bool,
    min_idle_minutes: u64,
    screen_off_since: Option<i64>,
) -> bool {
    if !screen_is_off {
        return false;
    }
    let min_secs = min_idle_minutes.saturating_mul(60);
    screen_off_elapsed(screen_off_since).is_some_and(|elapsed| elapsed >= min_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_duration_is_enforced() {
        let now = Utc::now().timestamp();
        // Turned off 1 minute ago, requirement 10 minutes.
        assert!(!screen_pass(true, 10, Some(now - 60)));
        // Turned off 11 minutes ago, requirement 10 minutes.
        assert!(screen_pass(true, 10, Some(now - 660)));
        // Never tracked: unknown duration must bias toward skipping.
        assert!(!screen_pass(true, 10, None));
        // Exactly the required duration passes.
        assert!(screen_pass(true, 10, Some(now - 600)));
        // Screen on: always blocked regardless of duration.
        assert!(!screen_pass(false, 10, Some(now - 600)));
        // min_idle_minutes = 0 means "screen off is enough".
        assert!(screen_pass(true, 0, Some(now - 1)));
    }
}
