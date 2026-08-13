# StorageRefresh-Rust

This is an independent open-source implementation of the general concept of background storage maintenance. It is not affiliated with, endorsed by, or identical to Xiaomi's proprietary Storage Refresh feature.

StorageRefresh-Rust is an independent, open-source Android root module that performs periodic, safe storage maintenance on the device's internal flash storage.

## Features
- Runs filesystem-level storage maintenance (TRIM/discard) using FITRIM ioctl.
- Executes under safe, idle conditions (screen off for a configurable duration, charging, cool temp, not actively used).
- Avoids interrupting active device usage.
- Supports both Magisk and KernelSU (same `/data/adb/modules` layout).
- Completely systemless module.
- Fails safe on unknown or unsupported filesystems.
- KernelSU: **Action button** + **WebUI** to trigger a manual maintenance cycle and inspect status/logs.
- Watchdog keeps the daemon alive across crashes.
- Graceful shutdown on both SIGINT and SIGTERM.

## Installation
1. Flash `StorageRefresh-Rust.zip` in Magisk Manager or KernelSU Manager.
2. Reboot your device.
3. The daemon will automatically start running in the background.

## Configuration
The default configuration is located at `/data/adb/storagerefresh/config.toml` after installation. You can edit this file to adjust intervals, idle requirements, and battery requirements.

```toml
[schedule]
min_interval_hours = 24
poll_interval_minutes = 20

[battery]
require_charging = true
min_capacity_percent = 30
max_temperature_c = 40.0
charging_override_capacity = 80

[screen]
min_idle_minutes = 10

[storage]
min_trim_len_mb = 4
allowed_fs = ["ext4", "f2fs"]
mount_point = "/data"

[safety]
require_foreground_check = true
dry_run = false
foreground_denylist = ["game", "camera", "video", "youtube", "netflix"]
```

### Tunables
| Key | Default | Meaning |
|---|---|---|
| `schedule.min_interval_hours` | 24 | Minimum hours between maintenance runs (clamped 6–168) |
| `schedule.poll_interval_minutes` | 20 | How often the daemon checks conditions (clamped 1–1440) |
| `battery.require_charging` | true | Only TRIM while charging |
| `battery.min_capacity_percent` | 30 | Minimum battery level (clamped 1–100) |
| `battery.max_temperature_c` | 40.0 | Maximum battery temperature |
| `battery.charging_override_capacity` | 80 | Allow TRIM while not charging above this capacity |
| `screen.min_idle_minutes` | 10 | Screen must be off continuously for this long (clamped 1–10080) |
| `storage.mount_point` | "/data" | Mount point to TRIM |
| `storage.allowed_fs` | ["ext4","f2fs"] | Filesystems TRIM is allowed on |
| `storage.min_trim_len_mb` | 4 | Minimum contiguous free length to trim |
| `safety.require_foreground_check` | true | Block TRIM while a denylisted app is foreground |
| `safety.dry_run` | false | Log what would be done without actually trimming |
| `safety.foreground_denylist` | [...] | Package-name substrings that block TRIM |

Older config files that lack the new keys keep working: missing values fall back to defaults, and every value is clamped to a safe range.

## KernelSU Action button & WebUI
After installing on KernelSU, open the module in KernelSU Manager:
- Tap the **Action** button to trigger one maintenance cycle immediately.
- Open the module **WebUI** to see daemon status, current conditions, last-run info, environment, and the latest log lines. The WebUI also has a **Run now** button.

The WebUI requires a recent KernelSU Manager (WebUI support). Permissions for `webroot/` are set automatically by KernelSU; the module never touches them.

## Logs
Logs are kept at `/data/local/tmp/StorageRefresh`. They rotate daily and only the last 7 files are kept. Log files and the directory are world-readable for easy debugging.
State file is located at `/data/adb/storagerefresh/state.json` (written atomically). The daemon PID is stored at `/data/adb/storagerefresh/daemon.pid`.

## Uninstallation
Disable or remove the module from your Root Manager.
During uninstallation, the log directory `/data/local/tmp/StorageRefresh` is automatically deleted and the daemon is stopped.
If you want to clear your config and state files completely, you can create a file:
`touch /data/adb/storagerefresh/remove_on_uninstall` before uninstalling.

## License
Apache License, Version 2.0.

### Building from source

To build the release binary targeted for Android, we use the Android NDK to cross-compile and link against Bionic libc correctly.
You will need the Android NDK installed and `rustup` with the `aarch64-linux-android` target installed.

```bash
# Provide the path to the NDK via the ANDROID_NDK_HOME variable
export ANDROID_NDK_HOME=/path/to/android-ndk

# The default build targets Android 14 (API Level 34).
# It will use the NDK's clang compiler, strip the binary using the NDK's llvm-strip,
# and package the Magisk/KernelSU module zip:
./build.sh
```

A PowerShell equivalent `build.ps1` is provided for Windows developers:
```powershell
$env:ANDROID_NDK_HOME="C:\path\to\android-ndk"
.\build.ps1
```
