#!/system/bin/sh
DATADIR="/data/adb/storagerefresh"

# Stop the running daemon
pkill -f storagerefresh-rust

# Only remove logs/state if explicitly requested via a flag file
if [ -f "$DATADIR/remove_on_uninstall" ]; then
  rm -rf "$DATADIR"
fi
