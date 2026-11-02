#!/bin/bash
# ArkOS > Options > Tools > Slot: try slot without changing anything.
# Power off or Restart inside slot brings you back here.
# ArkOS opens Tools through the /opt/system/Tools symlink: resolve it before
# going up, or ".." lands in /opt/system instead of on the card.
ROOT="$(dirname "$(cd -P "$(dirname "$0")" && pwd)")"
for r in "$ROOT" /roms /roms2; do
	[ -f "$r/slot/System/slot" ] && ROOT="$r" && break
done
[ "$(id -u)" -eq 0 ] || exec sudo -n bash "$0" "$@"
if [ ! -f "$ROOT/slot/System/r36s-launch.sh" ]; then
	echo "slot is not on the card (looked in $ROOT, /roms, /roms2)." > /dev/tty1 2>/dev/null
	sleep 6; exit 1
fi
exec sh "$ROOT/slot/System/r36s-launch.sh" test
