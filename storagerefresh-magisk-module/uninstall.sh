#!/system/bin/sh
DATADIR="/data/adb/storagerefresh"

# Stop the running daemon
pkill -f storagerefresh-rust

# Delete the log directory explicitly as per requirement
rm -rf "/data/local/tmp/StorageRefresh"

# Only remove config/state if explicitly requested via a flag file
if [ -f "$DATADIR/remove_on_uninstall" ]; then
  rm -rf "$DATADIR"
fi
