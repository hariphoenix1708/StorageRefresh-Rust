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

pub fn detect_storage() -> Vec<StorageInfo> {
    let mut mounts = Vec::new();

    // Fallback to /proc/mounts
    let mount_file = match File::open("/proc/self/mountinfo") {
        Ok(f) => f,
        Err(_) => match File::open("/proc/mounts") {
            Ok(f) => f,
            Err(_) => return mounts,
        }
    };

    let reader = BufReader::new(mount_file);

    // We mainly care about /data, but let's parse and filter.
    for line in reader.lines().filter_map(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Simple /proc/mounts parse: [device, mount_point, fs_type, options, ...]
        // mountinfo is a bit more complex, but standard fields are somewhat aligned or we can just do a dumb parse for this scope.
        // Let's assume standard /proc/mounts format or close enough for our needed fields.
        // If it's mountinfo, it's: [mount_id, parent_id, major:minor, root, mount_point, mount_options, optional_fields..., "-", fs_type, mount_source, super_options]

        let (device, mount_point, fs_type, options) = if line.contains(" - ") {
            // Likely mountinfo
            let mut parts = line.split(" - ");
            let pre = parts.next().unwrap_or("").split_whitespace().collect::<Vec<_>>();
            let post = parts.next().unwrap_or("").split_whitespace().collect::<Vec<_>>();

            if pre.len() >= 5 && post.len() >= 2 {
                (post[1], pre[4], post[0], pre[5])
            } else {
                continue;
            }
        } else {
            // Standard mounts
            if parts.len() >= 4 {
                (parts[0], parts[1], parts[2], parts[3])
            } else {
                continue;
            }
        };

        // We only care about /data for this specific tool typically, but let's just grab supported ones.
        if mount_point == "/data" {
            let is_readonly = options.contains("ro,") || options.starts_with("ro");
            let is_supported = fs_type == "ext4" || fs_type == "f2fs";

            mounts.push(StorageInfo {
                mount_point: mount_point.to_string(),
                fs_type: fs_type.to_string(),
                is_supported,
                is_readonly,
                block_device: device.to_string(),
            });
        }
    }

    mounts
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_parse_logic() {
        // Just checking compiling for now since we rely on /proc
    }
}
