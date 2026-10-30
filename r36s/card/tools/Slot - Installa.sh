#!/bin/bash
# Make slot what the R36S boots into, in place of EmulationStation.
# Undo with "Slot - Disinstalla", or put a file named ARKOS in EASYROMS/slot
# from a PC to boot ArkOS once without uninstalling.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[ "$(id -u)" -eq 0 ] || exec sudo -n bash "$0" "$@"
SD="$ROOT/slot"
LOG="$SD/Logs/install.log"
TTY=/dev/tty1
mkdir -p "$SD/Logs"
say() { echo "$*" | tee -a "$LOG" > "$TTY" 2>/dev/null; }

: > "$LOG"
sh "$SD/System/r36s-diag.sh" > "$SD/Logs/diag.txt" 2>&1
printf '\033c' > "$TTY" 2>/dev/null

if ! systemctl cat emulationstation.service > /dev/null 2>&1; then
	say "Non trovo emulationstation.service: installazione annullata."
	say "Manda $SD/Logs/diag.txt a chi ti aiuta col port."
	sleep 6; exit 1
fi
if [ ! -x "$SD/System/slot" ]; then
	say "Manca $SD/System/slot: installazione annullata."
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

systemctl daemon-reload
say "Slot installato. Riavvio tra 3 secondi..."
sync; sleep 3
reboot
