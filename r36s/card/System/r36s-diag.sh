#!/bin/sh
# Describe this ArkOS install, so the port can be checked from a PC.
section() { printf '\n=== %s\n' "$*"; }

if [ "${1:-}" = late ]; then
	section uptime; cut -d' ' -f1 /proc/uptime
	section boot time; systemd-analyze 2>&1
	systemd-analyze blame 2>&1 | head -40
	systemd-analyze critical-chain emulationstation.service 2>&1
	journalctl -b -o short-monotonic --no-pager 2>/dev/null | grep -iE 'tty1|console|cursor|ArkOS|started|finished' | head -120
	exit
fi

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
section boot
cut -d' ' -f1 /proc/uptime
cat /boot/boot.ini 2>&1 | grep bootargs
ps -eo pid,ppid,etimes,args 2>/dev/null | grep -v ' \['
echo "--- splash"
ls -la /etc/alternatives/default.plymouth /usr/share/plymouth/themes 2>&1
cat /etc/issue 2>&1
echo "--- ArkOS boot services"
for u in 351mp ddtbcheck dmesg setvtrgb wifi_importer ondemand oga_events console-setup; do
	systemctl cat "$u.service" 2>&1 | grep -v '^$' | head -25
	for f in $(systemctl show -p ExecStart -p ExecStartPre --value "$u.service" 2>/dev/null | grep -o 'path=[^ ;]*' | cut -d= -f2 | sort -u); do
		grep -Iq . "$f" 2>/dev/null || continue
		echo "--- $f"; head -80 "$f"
	done
done
crontab -l -u root 2>&1
systemctl list-unit-files --state=enabled --no-pager 2>&1
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
