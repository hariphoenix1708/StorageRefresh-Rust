#!/system/bin/sh
# Runs when the module is uninstalled/removed by Magisk or KernelSU.
MODDIR=${0%/*}
DATADIR="/data/adb/storagerefresh"
LOGDIR="/data/local/tmp/StorageRefresh"
PIDFILE="$DATADIR/daemon.pid"
BIN="$MODDIR/system/bin/storagerefresh-rust"

# Stop the daemon and make sure the watchdog cannot respawn it.
pkill -f "storagerefresh-rust --config" 2>/dev/null
if [ -f "$BIN" ]; then
  rm -f "$BIN" 2>/dev/null
fi
rm -f "$PIDFILE" 2>/dev/null

# Delete the log directory explicitly as per requirement.
rm -rf "$LOGDIR"

# Only remove config/state if explicitly requested via a flag file.
if [ -f "$DATADIR/remove_on_uninstall" ]; then
  rm -rf "$DATADIR"
fi

exit 0
