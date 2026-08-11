use chrono::Utc;

pub fn should_run(last_run_timestamp: i64, min_interval_hours: u64) -> bool {
    let now = Utc::now().timestamp();
    let min_interval_secs = (min_interval_hours * 3600) as i64;

    if last_run_timestamp == 0 {
        return true; // Never run
    }

    now - last_run_timestamp >= min_interval_secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_run() {
        let now = Utc::now().timestamp();
        assert!(should_run(0, 24)); // never run
        assert!(!should_run(now - 3600, 24)); // 1 hour ago, min 24
        assert!(should_run(now - (25 * 3600), 24)); // 25 hours ago, min 24
    }
}
