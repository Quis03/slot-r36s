#!/bin/sh
# Boot straight from the ArkOS logo into slot: no plymouth text splash (it
# leaves the console cursor on when it quits), no kernel text, and no wait
# for the boot video ArkOS starts before the frontend even without a video.
# Idempotent. Slot's launcher runs it on every boot, so an ArkOS update that
# rewrites boot.ini gets it back; "Slot - Disinstalla" undoes it.
BOOTINI=/boot/boot.ini
PARAMS="loglevel=0 vt.global_cursor_default=0 plymouth.enable=0"
BOOTARGS='^setenv bootargs "root='
VIDEO=/etc/systemd/system/play-video.service.d

if [ -w "$BOOTINI" ]; then
	for p in $PARAMS; do
		grep "$BOOTARGS" "$BOOTINI" | grep -q " $p" && continue
		[ -e "$BOOTINI.slot-bak" ] || cp -p "$BOOTINI" "$BOOTINI.slot-bak"
		sed -i "/$BOOTARGS/ s/\"\$/ $p\"/" "$BOOTINI" && echo "boot.ini: added $p"
	done
fi
if [ ! -f "$VIDEO/10-slot.conf" ]; then
	mkdir -p "$VIDEO"
	printf '[Unit]\nConditionPathExists=/boot/logo.mp4\n' > "$VIDEO/10-slot.conf"
	systemctl daemon-reload
	echo "play-video: runs only when /boot/logo.mp4 exists"
fi
