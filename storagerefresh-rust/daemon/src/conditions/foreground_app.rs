use std::process::Command;

pub fn check_foreground_app() -> bool {
    let output = Command::new("dumpsys")
        .arg("activity")
        .arg("activities")
        .output()
        .ok();

    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        // Look for mResumedActivity
        // Simple denylist check
        let denylist = vec!["game", "camera", "video", "youtube", "netflix"];

        for line in stdout.lines() {
            if line.contains("mResumedActivity") {
                let lower = line.to_lowercase();
                for bad in &denylist {
                    if lower.contains(bad) {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    // Fail safe
    false
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_foreground_mock() {}
}
