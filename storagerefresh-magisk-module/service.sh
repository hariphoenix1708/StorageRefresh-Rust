#!/system/bin/sh
# Starts the StorageRefresh-Rust daemon after boot and keeps it alive with a
# lightweight watchdog. Works identically on Magisk and KernelSU (both mount
# the module at /data/adb/modules/<id>).
MODDIR=${0%/*}
DATADIR="/data/adb/storagerefresh"
BIN="$MODDIR/system/bin/storagerefresh-rust"
LOGDIR="/data/local/tmp/StorageRefresh"
PIDFILE="$DATADIR/daemon.pid"

# Wait for boot completion before doing anything.
until [ "$(getprop sys.boot_completed)" = "1" ]; do
  sleep 5
done

# Ensure config + log dirs exist.
mkdir -p "$DATADIR" "$LOGDIR"
chmod 0777 "$LOGDIR"

# Seed the config from the default shipped with the module on first boot.
if [ ! -f "$DATADIR/config.toml" ]; then
  if [ -f "$MODDIR/system/etc/storagerefresh/config.toml.default" ]; then
    cp "$MODDIR/system/etc/storagerefresh/config.toml.default" "$DATADIR/config.toml"
  fi
fi

# Watchdog: launch the daemon, and respawn it if it exits or crashes while the
# module is still enabled. The uninstall script deletes the binary and pidfile
# so this loop goes quiet.
while true; do
  if [ -f "$BIN" ] && [ ! -f "$MODDIR/disable" ] && ! pgrep -f "storagerefresh-rust --config" >/dev/null 2>&1; then
    "$BIN" --config "$DATADIR/config.toml" --state "$DATADIR/state.json" \
      --log-dir "$LOGDIR" --pidfile "$PIDFILE" >/dev/null 2>&1
  fi
  sleep 15
done
