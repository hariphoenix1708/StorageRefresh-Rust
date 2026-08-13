use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Clone)]
pub struct StorageInfo {
    pub mount_point: String,
    pub fs_type: String,
    pub is_supported: bool,
    pub is_readonly: bool,
    pub block_device: String,
}

const SUPPORTED_FS: &[&str] = &["ext2", "ext3", "ext4", "f2fs", "xfs", "btrfs"];

pub fn detect_storage() -> Vec<StorageInfo> {
    let mut mounts = Vec::new();

    let mount_file = match File::open("/proc/self/mountinfo") {
        Ok(f) => f,
        Err(_) => match File::open("/proc/mounts") {
            Ok(f) => f,
            Err(_) => return mounts,
        },
    };

    let reader = BufReader::new(mount_file);

    // Prefer mountinfo; fall back to a plain /proc/mounts parse.
    for line in reader.lines().map_while(Result::ok) {
        let (device, mount_point, fs_type, options) = if line.contains(" - ") {
            // mountinfo: [mount_id, parent_id, major:minor, root, mount_point,
            // mount_options, optional..., "-", fs_type, mount_source, super_options]
            let mut parts = line.split(" - ");
            let pre = parts
                .next()
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>();
            let post = parts
                .next()
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>();

            if pre.len() >= 5 && post.len() >= 2 {
                (post[1], pre[4], post[0], pre[5])
            } else {
                continue;
            }
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                (parts[0], parts[1], parts[2], parts[3])
            } else {
                continue;
            }
        };

        let is_readonly = options.starts_with("ro") || options.contains(",ro");
        let is_supported = SUPPORTED_FS.contains(&fs_type);

        mounts.push(StorageInfo {
            mount_point: mount_point.to_string(),
            fs_type: fs_type.to_string(),
            is_supported,
            is_readonly,
            block_device: device.to_string(),
        });
    }

    mounts
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mountinfo_line_parsing() {
        // Reproduces the split logic without touching /proc.
        let line = "36 32 0:27 / /data rw,seclabel,nosuid,nodev,noatime - f2fs /dev/block/bootdevice/by-name/userdata rw";
        let mut parts = line.split(" - ");
        let pre = parts.next().unwrap().split_whitespace().collect::<Vec<_>>();
        let post = parts.next().unwrap().split_whitespace().collect::<Vec<_>>();

        let device = post[1];
        let mount_point = pre[4];
        let fs_type = post[0];
        let options = pre[5];

        assert_eq!(device, "/dev/block/bootdevice/by-name/userdata");
        assert_eq!(mount_point, "/data");
        assert_eq!(fs_type, "f2fs");
        assert!(options.starts_with("rw"));
    }

    #[test]
    fn test_readonly_detection() {
        assert!(is_readonly("ro"));
        assert!(is_readonly("ro,nosuid"));
        assert!(is_readonly("rw,ro,noatime"));
        assert!(!is_readonly("rw,nosuid"));
    }

    fn is_readonly(options: &str) -> bool {
        options.starts_with("ro") || options.contains(",ro")
    }
}
