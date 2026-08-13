use std::process::Command;

#[test]
fn test_binary_prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_storagerefresh-rust"))
        .arg("--version")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("storagerefresh-rust"));
}

#[test]
fn test_binary_detect_only_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_storagerefresh-rust"))
        .arg("--detect-only")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Environment:"));
    assert!(stdout.contains("Storage:"));
}

#[test]
fn test_binary_status_prints_valid_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_storagerefresh-rust"))
        .args([
            "--status",
            "--config",
            "/nonexistent/config.toml",
            "--state",
            "/nonexistent/state.json",
        ])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--status output must be valid JSON");
    assert!(parsed.get("conditions").is_some());
    assert!(parsed.get("state").is_some());
    assert!(parsed.get("running").is_some());
}
