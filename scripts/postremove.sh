#!/bin/bash
set -e

case "$1" in
    purge)
        rm -f /etc/udev/rules.d/45-ddcutil-i2c.rules
        udevadm control --reload-rules 2>/dev/null || true
        update-desktop-database -q /usr/share/applications 2>/dev/null || true
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
        ;;
esac

exit 0