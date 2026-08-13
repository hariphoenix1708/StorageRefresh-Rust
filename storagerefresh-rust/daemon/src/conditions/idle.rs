use std::process::Command;

use crate::conditions::screen::screen_off_elapsed;

/// True when the device reports deep idle, OR (fallback) the tracked
/// screen-off duration already satisfies the idle requirement.
/// Fails safe: if neither can be proven, returns false (bias toward skipping).
pub fn check_idle(min_idle_minutes: u64, screen_off_since: Option<i64>) -> bool {
    if let Ok(out) = Command::new("dumpsys")
        .args(["deviceidle", "get", "deep"])
        .output()
    {
        if out.status.success() {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_uppercase();
            if state == "IDLE" {
                return true;
            }
        }
    }

    // Fallback: if the device isn't in deep idle yet, rely on the tracked
    // screen-off duration so `min_idle_minutes` is still honored.
    let min_secs = min_idle_minutes.saturating_mul(60);
    screen_off_elapsed(screen_off_since).is_some_and(|elapsed| elapsed >= min_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_fallback_honors_duration() {
        let now = chrono::Utc::now().timestamp();
        assert!(check_idle(10, Some(now - 700)));
        assert!(!check_idle(10, Some(now - 60)));
        assert!(!check_idle(10, None));
    }
}
