#!/system/bin/sh
MODDIR=${0%/*}

# Wait for boot completion
until [ "$(getprop sys.boot_completed)" = "1" ]; do
  sleep 5
done

# Ensure config dir exists
DATADIR="/data/adb/storagerefresh"
mkdir -p "$DATADIR"

# Copy default config if it doesn't exist
if [ ! -f "$DATADIR/config.toml" ]; then
  cp "$MODDIR/system/etc/storagerefresh/config.toml.default" "$DATADIR/config.toml"
fi

# Launch daemon in background
"$MODDIR/system/bin/storagerefresh-rust" --config "$DATADIR/config.toml" --state "$DATADIR/state.json" --log-dir "$DATADIR/logs" >/dev/null 2>&1 &
