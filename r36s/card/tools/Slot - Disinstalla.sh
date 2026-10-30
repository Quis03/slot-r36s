#!/bin/bash
# Put EmulationStation back as what the R36S boots into. Keeps EASYROMS/slot.
[ "$(id -u)" -eq 0 ] || exec sudo -n bash "$0" "$@"
rm -f /etc/systemd/system/emulationstation.service.d/10-slot.conf
rmdir /etc/systemd/system/emulationstation.service.d 2>/dev/null
rm -f /etc/systemd/logind.conf.d/10-slot.conf
if [ -f /etc/slot-disabled-services ]; then
	sort -u /etc/slot-disabled-services | xargs -r systemctl enable
	rm -f /etc/slot-disabled-services
fi
# the boot tweaks of System/r36s-boot.sh
for p in loglevel=0 vt.global_cursor_default=0 plymouth.enable=0; do
	sed -i "/^setenv bootargs \"root=/ s/ $p//" /boot/boot.ini
done
rm -rf /etc/systemd/system/play-video.service.d
systemctl daemon-reload
echo "Slot disinstallato, torna ArkOS. Riavvio..." > /dev/tty1 2>/dev/null
sync; sleep 3
reboot
