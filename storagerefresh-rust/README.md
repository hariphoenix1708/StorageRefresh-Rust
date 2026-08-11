# StorageRefresh-Rust

This is an independent open-source implementation of the general concept of background storage maintenance. It is not affiliated with, endorsed by, or identical to Xiaomi's proprietary Storage Refresh feature.

StorageRefresh-Rust is an independent, open-source Android root module that performs periodic, safe storage maintenance on the device's internal flash storage.

## Features
- Runs filesystem-level storage maintenance (TRIM/discard) using FITRIM ioctl.
- Executes under safe, idle conditions (screen off, charging, cool temp, not actively used).
- Avoids interrupting active device usage.
- Supports both Magisk and KernelSU.
- Completely systemless module.
- Fails safe on unknown or unsupported filesystems.

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

[screen]
min_idle_minutes = 10

[storage]
min_trim_len_mb = 4
allowed_fs = ["ext4", "f2fs"]

[safety]
require_foreground_check = true
dry_run = false
```

## Logs
Logs are kept at `/data/adb/storagerefresh/logs`. They rotate daily and only the last 7 files are kept.
State file is located at `/data/adb/storagerefresh/state.json`.

## Uninstallation
Disable or remove the module from your Root Manager.
If you want to clear your logs/config completely, you can create a file:
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
# and package the Magisk module zip:
./build.sh
```

A PowerShell equivalent `build.ps1` is provided for Windows developers:
```powershell
$env:ANDROID_NDK_HOME="C:\path\to\android-ndk"
.\build.ps1
```
