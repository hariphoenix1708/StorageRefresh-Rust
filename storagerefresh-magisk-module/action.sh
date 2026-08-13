#!/system/bin/sh
# Runs when the user taps the module Action button in the KernelSU Manager.
# Triggers one maintenance cycle (or reports status if the daemon is already
# running) and stores the result for the WebUI to display.
MODDIR=${0%/*}
DATADIR="/data/adb/storagerefresh"
BIN="$MODDIR/system/bin/storagerefresh-rust"
LOGDIR="/data/local/tmp/StorageRefresh"
PIDFILE="$DATADIR/daemon.pid"
RESULT="$DATADIR/action_result.json"

mkdir -p "$DATADIR" "$LOGDIR"

if [ ! -f "$BIN" ]; then
  echo '{"ok":false,"error":"storagerefresh-rust binary missing"}' > "$RESULT"
  echo '! Binary missing'
  exit 1
fi

if [ ! -f "$DATADIR/config.toml" ]; then
  if [ -f "$MODDIR/system/etc/storagerefresh/config.toml.default" ]; then
    cp "$MODDIR/system/etc/storagerefresh/config.toml.default" "$DATADIR/config.toml"
  fi
fi

# If the daemon is already running, don't run a second cycle concurrently;
# just report current status instead.
if pgrep -f "storagerefresh-rust --config" >/dev/null 2>&1; then
  "$BIN" --status --config "$DATADIR/config.toml" --state "$DATADIR/state.json" > "$RESULT" 2>&1
  echo "Daemon is running; showing current status (see WebUI)."
  exit 0
fi

# Run one maintenance cycle synchronously so the Action result reflects it.
"$BIN" --once --config "$DATADIR/config.toml" --state "$DATADIR/state.json" \
  --log-dir "$LOGDIR" --pidfile "$PIDFILE" > "$RESULT" 2>&1
status=$?

"$BIN" --status --config "$DATADIR/config.toml" --state "$DATADIR/state.json" >> "$RESULT" 2>&1

if [ "$status" -eq 0 ]; then
  echo "Maintenance cycle finished. Result stored in $RESULT"
else
  echo "! Maintenance cycle failed (exit $status). See $RESULT"
fi
exit "$status"
