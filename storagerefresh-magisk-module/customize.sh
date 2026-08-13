#!/system/bin/sh
# Post-install permission fix.
# Runs on BOTH Magisk and KernelSU. `$MODPATH` is exported by both installers;
# fall back to ${0%/*} when sourced in an environment without it.
MODDIR="${MODPATH:-${0%/*}}"

# system/bin is made 0755 automatically by both installers, but keep this as a
# safety net in case a future installer changes that behavior.
if [ -f "$MODDIR/system/bin/storagerefresh-rust" ]; then
  chmod 0755 "$MODDIR/system/bin/storagerefresh-rust" 2>/dev/null
fi

# Root-level scripts are 0644 by default; give them exec bits so they behave
# identically on Magisk and KernelSU.
for script in service.sh uninstall.sh action.sh; do
  if [ -f "$MODDIR/$script" ]; then
    chmod 0755 "$MODDIR/$script" 2>/dev/null
  fi
done

exit 0
