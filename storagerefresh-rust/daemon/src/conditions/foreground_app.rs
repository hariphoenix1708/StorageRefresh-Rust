use std::process::Command;

/// Pure decision logic on a single `mResumedActivity`/`topResumedActivity`
/// line. Returns true when the resumed activity is not on the denylist.
pub fn foreground_allowed(line: &str, denylist: &[String]) -> bool {
    let lower = line.to_lowercase();
    !denylist
        .iter()
        .any(|bad| !bad.is_empty() && lower.contains(&bad.to_lowercase()))
}

/// True when the foreground app is not on the denylist. Runs `dumpsys
/// activity activities`, which is heavy, so failures must bias toward
/// skipping (returns false when the state cannot be determined).
pub fn check_foreground_app(denylist: &[String]) -> bool {
    let output = Command::new("dumpsys")
        .args(["activity", "activities"])
        .output()
        .ok();

    let Some(out) = output else {
        return false; // Fail safe
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);

    for line in stdout.lines() {
        if (line.contains("mResumedActivity") || line.contains("topResumedActivity"))
            && !foreground_allowed(line, denylist)
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn denylist() -> Vec<String> {
        vec!["game".to_string(), "camera".to_string()]
    }

    #[test]
    fn test_denylist_matches_ignoring_case() {
        let dl = denylist();
        assert!(!foreground_allowed(
            "mResumedActivity: ActivityRecord{abc u0 com.foo.GameApp/.Main}",
            &dl
        ));
        assert!(!foreground_allowed(
            "mResumedActivity: ActivityRecord{abc u0 com.foo.Camera360/.Main}",
            &dl
        ));
        assert!(!foreground_allowed(
            "topResumedActivity: ActivityRecord{abc u0 com.foo.game/.Activity}",
            &dl
        ));
    }

    #[test]
    fn test_safe_app_passes() {
        let dl = denylist();
        assert!(foreground_allowed(
            "mResumedActivity: ActivityRecord{abc u0 com.android.settings/.Main}",
            &dl
        ));
        assert!(foreground_allowed("", &dl));
    }
}
