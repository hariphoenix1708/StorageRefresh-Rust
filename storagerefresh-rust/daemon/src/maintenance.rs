use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::process::Command;

use log::{info, warn};

// FITRIM ioctl setup
// In C: #define FITRIM _IOWR('X', 121, struct fstrim_range)
// Nix provides ioctl! macro





#[repr(C)]
#[derive(Debug, Default)]
pub struct FstrimRange {
    pub start: u64,
    pub len: u64,
    pub minlen: u64,
}

nix::ioctl_readwrite!(fitrim, b'X', 121, FstrimRange);

pub fn run_maintenance(dry_run: bool, min_trim_len_mb: u64) -> anyhow::Result<()> {
    if dry_run {
        info!("Dry run mode enabled. Would trim /data here.");
        return Ok(());
    }

    let min_len_bytes = min_trim_len_mb * 1024 * 1024;

    // Check if it's f2fs and if we should nudge it (optional phase 2)
    // For now we just run FITRIM on /data

    let path = "/data"; // Hardcode /data for now as required

    match run_fitrim_ioctl(path, min_len_bytes) {
        Ok(trimmed) => {
            info!("FITRIM ioctl succeeded on {}: {} bytes trimmed", path, trimmed);
            Ok(())
        },
        Err(e) => {
            warn!("FITRIM ioctl failed on {}: {}. Falling back to fstrim binary.", path, e);
            run_fstrim_bin(path)
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

    // unsafe block because we are calling an ioctl
    let res = unsafe { fitrim(fd, &mut range) };

    if let Err(e) = res {
        return Err(anyhow::anyhow!("ioctl failed: {}", e));
    }

    Ok(range.len)
}

fn run_fstrim_bin(path: &str) -> anyhow::Result<()> {
    let output = Command::new("fstrim")
        .arg("-v")
        .arg(path)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("fstrim binary success: {}", stdout.trim());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("fstrim binary failed with exit code {}: {}", output.status, stderr.trim()))
    }
}
