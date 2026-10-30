#!/bin/sh
# Describe this ArkOS install, so the port can be checked from a PC.
section() { printf '\n=== %s\n' "$*"; }

section date; date
section kernel; uname -a; cat /proc/cmdline
section os; cat /etc/os-release 2>/dev/null; ldd --version 2>&1 | head -1
section model; tr -d '\0' < /proc/device-tree/model 2>/dev/null; echo
section dri; ls -l /dev/dri 2>&1
section fb; cat /sys/class/graphics/fb0/modes /sys/class/graphics/fb0/virtual_size 2>&1
section gl libraries
ls -l /usr/lib/aarch64-linux-gnu/ 2>/dev/null | grep -E 'lib(EGL|GLES|gbm|mali|MALI|drm|asound)'
section input; cat /proc/bus/input/devices
section sysfs
for d in /sys/class/backlight/* /sys/class/power_supply/* /sys/class/leds/*; do
	printf '%s: ' "$d"; cat "$d/type" "$d/max_brightness" "$d/capacity" "$d/status" 2>/dev/null | tr '\n' ' '; echo
done
section clocks
for g in /sys/devices/system/cpu/cpufreq/policy*/scaling_governor /sys/class/devfreq/*/governor; do
	d="$(dirname "$g")"; echo "$g: $(cat "$g") of $(cat "${g%governor}available_governors" 2>/dev/null), now $(cat "$d/scaling_cur_freq" "$d/cur_freq" 2>/dev/null)"
done
section headphones
grep -iE 'hp|head|jack|spk' /sys/kernel/debug/gpio 2>&1
for e in /dev/input/event*; do
	grep -q Headphones "/sys/class/input/$(basename "$e")/device/name" 2>/dev/null || continue
	evtest --query "$e" EV_SW SW_HEADPHONE_INSERT; echo "$e headphone switch: $? (10 = plugged in)"
done
amixer sget 'Playback Path' 2>&1 | grep -E 'Items|Item0'
section audio; aplay -l 2>&1; amixer 2>&1; cat /etc/asound.conf 2>&1; aplay -L 2>&1 | head -20
section mounts; grep -E 'roms|mmcblk' /proc/mounts
section processes; ps -eo pid,user,pcpu,args 2>/dev/null | grep -v ' \['
section emulationstation service
systemctl cat emulationstation.service 2>&1 | head -40
systemctl is-enabled emulationstation.service 2>&1
section logind; grep -rh '^Handle\|^#Handle' /etc/systemd/logind.conf /etc/systemd/logind.conf.d 2>/dev/null
section sudo; id; sudo -n true 2>&1 && echo "sudo ok"
section arkos helpers
for f in /home/ark/.config/imageshift.sh /usr/local/bin/auto_suspend.py /usr/bin/mpv_sense \
	/usr/local/bin/checknswitchforusbdac.sh /usr/local/bin/wifi_icon_state_updater.sh \
	/usr/local/bin/batt_life_warning.py; do
	echo "--- $f"; head -60 "$f" 2>&1
done
echo "--- started from"
grep -rlE 'imageshift|auto_suspend|mpv_sense|checknswitchforusbdac|wifi_icon_state|batt_life_warning' \
	/etc/systemd/system /lib/systemd/system /etc/rc.local /etc/init.d /home/ark/.config/autostart \
	/usr/bin/emulationstation/emulationstation.sh /var/spool/cron 2>/dev/null
