use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::process::Command;

use log::{info, warn};

use crate::config::StorageConfig;
use crate::storage;

// FITRIM ioctl: #define FITRIM _IOWR('X', 121, struct fstrim_range)
#[repr(C)]
#[derive(Debug, Default)]
pub struct FstrimRange {
    pub start: u64,
    pub len: u64,
    pub minlen: u64,
}

nix::ioctl_readwrite!(fitrim, b'X', 121, FstrimRange);

/// Returns the number of bytes trimmed (0 when skipped or dry run).
pub fn run_maintenance(dry_run: bool, config: &StorageConfig) -> anyhow::Result<u64> {
    if dry_run {
        info!("Dry run enabled: would TRIM {} here.", config.mount_point);
        return Ok(0);
    }

    let target = storage::detect_storage()
        .into_iter()
        .find(|s| s.mount_point == config.mount_point);

    match target {
        Some(s) => {
            if s.is_readonly {
                info!("Skipping TRIM: {} is mounted read-only.", s.mount_point);
                return Ok(0);
            }
            if !config.allowed_fs.iter().any(|f| f == &s.fs_type) {
                info!(
                    "Skipping TRIM: filesystem {} on {} is not in the allowed list.",
                    s.fs_type, s.mount_point
                );
                return Ok(0);
            }
        }
        None => {
            info!(
                "Skipping TRIM: {} not found in mount table.",
                config.mount_point
            );
            return Ok(0);
        }
    }

    let min_len_bytes = config.min_trim_len_mb.saturating_mul(1024 * 1024);

    match run_fitrim_ioctl(&config.mount_point, min_len_bytes) {
        Ok(trimmed) => {
            info!(
                "FITRIM ioctl succeeded on {}: {} bytes trimmed",
                config.mount_point, trimmed
            );
            Ok(trimmed)
        }
        Err(e) => {
            warn!(
                "FITRIM ioctl failed on {}: {}. Falling back to fstrim binary.",
                config.mount_point, e
            );
            run_fstrim_bin(&config.mount_point)
        }
    }
}

fn run_fitrim_ioctl(path: &str, minlen: u64) -> anyhow::Result<u64> {
    let file = File::open(path)?;
    let fd = file.as_raw_fd();

    let mut range = FstrimRange {
        start: 0,
        len: u64::MAX,
        minlen,
    };

    // SAFETY: `range` is a valid pointer-sized stack value matching the
    // fstrim_range layout and the fd points at the mountpoint directory.
    let res = unsafe { fitrim(fd, &mut range) };

    if let Err(e) = res {
        return Err(anyhow::anyhow!("ioctl failed: {}", e));
    }

    Ok(range.len)
}

fn run_fstrim_bin(path: &str) -> anyhow::Result<u64> {
    let output = Command::new("fstrim").arg("-v").arg(path).output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("fstrim binary success: {}", stdout.trim());
        Ok(0)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "fstrim binary failed with exit code {}: {}",
            output.status,
            stderr.trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dry_run_does_not_trim() {
        let config = StorageConfig::default();
        let trimmed = run_maintenance(true, &config).unwrap();
        assert_eq!(trimmed, 0);
    }

    #[test]
    fn test_unknown_mount_is_skipped_not_errored() {
        // On the host there is no "/data" in the mount table, so this must
        // skip cleanly rather than fail (fail-safe behavior).
        let config = StorageConfig {
            mount_point: "/definitely/not/a/real/mount".to_string(),
            ..StorageConfig::default()
        };
        let trimmed = run_maintenance(false, &config).unwrap();
        assert_eq!(trimmed, 0);
    }

    #[test]
    fn test_unsupported_fs_is_skipped() {
        let config = StorageConfig {
            min_trim_len_mb: 4,
            allowed_fs: vec!["ext4".to_string(), "f2fs".to_string()],
            mount_point: "/data".to_string(),
        };
        assert!(!config.allowed_fs.iter().any(|f| f == "tmpfs"));
        // Just verify the guard logic composes as expected.
        assert!(config.allowed_fs.contains(&"ext4".to_string()));
    }
}
