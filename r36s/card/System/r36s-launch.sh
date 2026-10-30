#!/bin/sh
# Start slot on ArkOS for the R36S. Runs as root.
#
#   r36s-launch.sh test      run slot once; Power off / Restart in slot just
#                            quit back to whoever started us (ArkOS Tools)
#   r36s-launch.sh service   run as EmulationStation's replacement; falls back
#                            to EmulationStation if slot is missing, if the
#                            card holds a file named ARKOS, or if slot crashes
SYS="$(cd "$(dirname "$0")" && pwd)"
SD="$(dirname "$SYS")"
LOGS="$SD/Logs"
MODE="${1:-test}"
ES="${SLOT_ES:-/usr/bin/emulationstation/emulationstation.sh}"
ES_USER="${SLOT_ES_USER:-ark}"

mkdir -p "$LOGS" 2>/dev/null

log() {
	echo "$(date '+%F %T') $*" >> "$LOGS/slot-session.log" 2>/dev/null
}

arkos() {
	log "handing over to EmulationStation: $*"
	if [ -x "$ES" ]; then
		exec runuser -u "$ES_USER" -- "$ES"
	fi
	log "no EmulationStation at $ES"
	exit 1
}

if [ "$MODE" = service ]; then
	[ -e "$SD/ARKOS" ] || [ -e "$SD/ARKOS.txt" ] && arkos "ARKOS file on the card"
	[ -x "$SYS/slot" ] || arkos "no slot binary"
fi

export SLOT_ROOT="$SD"
export SLOT_RUN=/run
export BASEOS_TARGET=r36s
export BASEOS_MODEL="R36S"

if [ "$MODE" = test ]; then
	# slot powers off with poweroff(8) and restarts with reboot(8); in a test
	# run both just end slot, and ArkOS comes back.
	SHIMS=/tmp/slot-shims
	mkdir -p "$SHIMS"
	for cmd in poweroff reboot; do
		printf '#!/bin/sh\nkill -TERM %s 2>/dev/null\nexit 0\n' "\$PPID" > "$SHIMS/$cmd"
		chmod +x "$SHIMS/$cmd"
	done
	export PATH="$SHIMS:$PATH"
fi

sh "$SYS/r36s-diag.sh" > "$LOGS/diag.txt" 2>&1

# ArkOS' hotkey daemon also turns the volume on Vol+/-, and slot already does.
for d in ogage oga_events; do
	pkill -STOP -x "$d" 2>/dev/null && log "paused $d"
done

[ -f "$LOGS/slot.log" ] && mv -f "$LOGS/slot.log" "$LOGS/slot.log.1"
log "start $MODE"
"$SYS/slot" > "$LOGS/slot.log" 2>&1
status=$?
log "slot exited $status"

for d in ogage oga_events; do
	pkill -CONT -x "$d" 2>/dev/null
done
sync

if [ "$MODE" = service ] && [ "$status" -ne 0 ]; then
	arkos "slot exited $status"
fi
exit "$status"
