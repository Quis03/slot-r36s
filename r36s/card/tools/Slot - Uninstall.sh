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
systemctl daemon-reload
echo "slot uninstalled, ArkOS is back. Restarting..." > /dev/tty1 2>/dev/null
sync; sleep 3
reboot
