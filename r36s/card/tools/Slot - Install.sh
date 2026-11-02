#!/bin/bash
# Make slot what the R36S boots into, in place of EmulationStation.
# Undo with "Slot - Uninstall", or put a file named ARKOS in EASYROMS/slot
# from a PC to boot ArkOS once without uninstalling.
# ArkOS opens Tools through the /opt/system/Tools symlink: resolve it before
# going up, or ".." lands in /opt/system instead of on the card.
ROOT="$(dirname "$(cd -P "$(dirname "$0")" && pwd)")"
for r in "$ROOT" /roms /roms2; do
	[ -f "$r/slot/System/slot" ] && ROOT="$r" && break
done
[ "$(id -u)" -eq 0 ] || exec sudo -n bash "$0" "$@"
SD="$ROOT/slot"
# an older version of this script made /opt/system/slot by mistake
STRAY="$(cd "$(dirname "$0")/.." && pwd)/slot"
[ "$STRAY" != "$SD" ] && [ -d "$STRAY" ] && [ ! -e "$STRAY/System" ] && rm -rf "$STRAY"
LOG="$SD/Logs/install.log"
TTY=/dev/tty1
mkdir -p "$SD/Logs"
say() { echo "$*" | tee -a "$LOG" > "$TTY" 2>/dev/null; }

: > "$LOG"
sh "$SD/System/r36s-diag.sh" > "$SD/Logs/diag.txt" 2>&1
printf '\033c' > "$TTY" 2>/dev/null

if ! systemctl cat emulationstation.service > /dev/null 2>&1; then
	say "emulationstation.service not found: nothing installed."
	say "Attach $SD/Logs/diag.txt to an issue on github.com/Quis03/slot-r36s."
	sleep 6; exit 1
fi
if [ ! -f "$SD/System/slot" ]; then
	say "slot is not on the card (looked in $ROOT, /roms, /roms2): nothing installed."
	sleep 6; exit 1
fi

DROPIN=/etc/systemd/system/emulationstation.service.d
mkdir -p "$DROPIN"
cat > "$DROPIN/10-slot.conf" <<UNIT
[Unit]
RequiresMountsFor=$ROOT

[Service]
User=root
ExecStart=
ExecStart=/bin/sh $SD/System/r36s-launch.sh service
UNIT

# slot sleeps the screen on a short press of Power itself; stop logind
# suspending or powering off underneath it.
mkdir -p /etc/systemd/logind.conf.d
cat > /etc/systemd/logind.conf.d/10-slot.conf <<CONF
[Login]
HandlePowerKey=ignore
CONF

# Sharing and remote services slot never uses, and the ArkOS helpers that
# only serve EmulationStation: the Wi-Fi icon of its theme (nmcli twice a
# second), the buttons of its video player and its idle suspend. Remember
# which were on, so "Slot - Uninstall" turns the same ones back on.
STATE=/etc/slot-disabled-services
for unit in smbd nmbd ssh bluetooth ModemManager avahi-daemon cups \
	wifi-icon-updater mpv autosuspend; do
	[ "$(systemctl is-enabled "$unit" 2>/dev/null)" = enabled ] || continue
	systemctl disable --now "$unit" >> "$LOG" 2>&1 && echo "$unit" >> "$STATE"
	say "Turned off $unit"
done

systemctl daemon-reload
say "slot installed. Restarting in 3 seconds..."
sync; sleep 3
reboot
