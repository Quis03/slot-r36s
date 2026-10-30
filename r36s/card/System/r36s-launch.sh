#!/bin/sh
# Start slot on ArkOS for the R36S. Runs as root.
#
#   r36s-launch.sh test      run slot once; Power off / Restart in slot just
#                            quit back to whoever started us (ArkOS Tools)
#   r36s-launch.sh service   run as EmulationStation's replacement; falls back
#                            to EmulationStation if slot is missing, if the
#                            card holds a file named ARKOS, or if slot crashes
SYS="$(cd -P "$(dirname "$0")" && pwd)"
SD="$(dirname "$SYS")"
LOGS="$SD/Logs"
MODE="${1:-test}"
ES="${SLOT_ES:-/usr/bin/emulationstation/emulationstation.sh}"
ES_USER="${SLOT_ES_USER:-ark}"

mkdir -p "$LOGS" 2>/dev/null

# Until slot shows its first frame the console under it is on screen: keep
# it blank, without the blinking cursor.
if [ "$MODE" = service ]; then
	printf '\033[?25l\033[2J\033[H' > /dev/tty1 2>/dev/null
	echo 0 > /sys/class/graphics/fbcon/cursor_blink 2>/dev/null
fi

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
	[ -f "$SYS/slot" ] || arkos "no slot binary"
fi

export SLOT_ROOT="$SD"
export SLOT_RUN=/run
export BASEOS_TARGET=r36s
export BASEOS_MODEL="R36S"
# mGBA cannot hold full speed on the RK3326; gpSP's dynarec can. A game set
# to mGBA in slot's core picker keeps it.
export SLOT_GBA_CORE=gpsp
# Start each frame right after the last flip: the KMS swap also waits on the
# GPU, which the late start's 1.5 ms margin does not cover.
export SLOT_PACER=off

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

# A file named TRACE in EASYROMS/slot turns on slot's own traces (every
# button, audio pacing, input latency). They are off otherwise: while the
# log grows, slot syncs the card every second, and that costs frames.
TRACE=
[ -e "$SD/TRACE" ] || [ -e "$SD/TRACE.txt" ] && TRACE=1
if [ "$MODE" = test ] || [ -n "$TRACE" ]; then
	sh "$SYS/r36s-diag.sh" > "$LOGS/diag.txt" 2>&1
fi

# ArkOS' hotkey daemon also turns the volume on Vol+/-, and slot already does.
for d in ogage oga_events; do
	pkill -STOP -x "$d" 2>/dev/null && log "paused $d"
done

# slot sets the volume itself, in software, so the RK817 codec has to be
# routed to the speaker and left at full level while it runs. ArkOS' own
# levels come back when slot exits.
MIXER=/tmp/slot-asound.state
alsactl store -f "$MIXER" 2>/dev/null || rm -f "$MIXER"
log "mixer before: path $(amixer sget 'Playback Path' 2>&1 | grep -o "Item0: '[^']*'"), $(amixer sget Playback 2>&1 | grep -m1 -o '\[[0-9]*%\]')"
case "$(amixer sget 'Playback Path' 2>/dev/null | grep Item0)" in
*"'OFF'"*|*"'RCV'"*) amixer -q sset 'Playback Path' SPK ;;
esac
amixer -q sset Playback 100% unmute 2>/dev/null || amixer -q sset Playback 100%
log "mixer for slot: path $(amixer sget 'Playback Path' 2>&1 | grep -o "Item0: '[^']*'"), $(amixer sget Playback 2>&1 | grep -m1 -o '\[[0-9]*%\]')"

# ArkOS keeps the CPU, GPU and memory clocks low outside games and raises
# them only when it starts an emulator. In slot a game can start at any time,
# so run at full clocks throughout; the governors come back when slot exits.
CLOCKS=/tmp/slot-governors
: > "$CLOCKS"
for g in /sys/devices/system/cpu/cpufreq/policy*/scaling_governor /sys/class/devfreq/*/governor; do
	[ -w "$g" ] && grep -qw performance "${g%governor}available_governors" 2>/dev/null || continue
	echo "$g $(cat "$g")" >> "$CLOCKS"
	echo performance > "$g"
done
log "clocks to performance: $(awk '{printf "%s %s; ", $1, $2}' "$CLOCKS")"

if [ -n "$TRACE" ]; then
	export SLOT_TRACE=1 SLOT_TRACE_LATENCY=1
	# systemd only times the boot once it is over
	(sleep 30; sh "$SYS/r36s-diag.sh" late > "$LOGS/boot-time.txt" 2>&1) &
fi

[ -f "$LOGS/slot.log" ] && mv -f "$LOGS/slot.log" "$LOGS/slot.log.1"
heat() {
	t=$(cat /sys/class/thermal/thermal_zone0/temp 2>/dev/null)
	f=$(cat /sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq 2>/dev/null)
	echo "cpu ${t%???}°C at ${f%???} MHz"
}
jack() {
	for e in /dev/input/event*; do
		grep -q Headphones "/sys/class/input/$(basename "$e")/device/name" 2>/dev/null || continue
		evtest --query "$e" EV_SW SW_HEADPHONE_INSERT >/dev/null 2>&1
		[ $? -eq 10 ] && echo "headphones in" || echo "headphones out"
		return
	done
	echo "no headphone switch"
}
log "start $MODE $(cut -d' ' -f1 /proc/uptime)s after power on, $(heat), $(jack)"
# exFAT keeps no exec bit of its own; if the mount gives none, load slot
# through the dynamic linker instead.
if [ -x "$SYS/slot" ]; then
	"$SYS/slot" > "$LOGS/slot.log" 2>&1
else
	log "slot is not executable on this mount, starting it through ld.so"
	/lib/ld-linux-aarch64.so.1 "$SYS/slot" > "$LOGS/slot.log" 2>&1
fi
status=$?
log "slot exited $status, $(heat), $(jack), path $(amixer sget 'Playback Path' 2>&1 | grep -o "Item0: '[^']*'")"

for d in ogage oga_events; do
	pkill -CONT -x "$d" 2>/dev/null
done
[ -f "$MIXER" ] && alsactl restore -f "$MIXER" 2>/dev/null
while read -r g gov; do
	echo "$gov" > "$g" 2>/dev/null
done < "$CLOCKS"
sync

if [ "$MODE" = service ] && [ "$status" -ne 0 ]; then
	arkos "slot exited $status"
fi
exit "$status"
