use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Serialize, Clone, Default, Deserialize)]
pub struct Environment {
    pub android_version: String,
    pub android_sdk: String,
    pub is_hyperos: bool,
    pub root_type: RootType,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default, Deserialize)]
pub enum RootType {
    Magisk,
    KernelSU,
    #[default]
    Unknown,
}

pub fn detect_environment() -> Environment {
    Environment {
        android_version: get_prop("ro.build.version.release"),
        android_sdk: get_prop("ro.build.version.sdk"),
        is_hyperos: is_hyperos(),
        root_type: detect_root(),
    }
}

fn get_prop(prop_name: &str) -> String {
    let output = Command::new("getprop").arg(prop_name).output().ok();

    if let Some(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    String::new()
}

fn is_hyperos() -> bool {
    !get_prop("ro.miui.ui.version.name").is_empty() || !get_prop("ro.mi.os.version.name").is_empty()
}

fn detect_root() -> RootType {
    if fs::metadata("/data/adb/magisk").is_ok() {
        RootType::Magisk
    } else if fs::metadata("/data/adb/ksu").is_ok() {
        RootType::KernelSU
    } else {
        RootType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hard to mock easily without traits/dependency injection, but we can verify our defaults.
    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert_eq!(env.android_version, "");
        assert_eq!(env.android_sdk, "");
        assert!(!env.is_hyperos);
        assert_eq!(env.root_type, RootType::Unknown);
    }
}
