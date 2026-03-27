#!/bin/bash
set -e

log() { echo "[sun] $1"; }

# Add i2c group if it doesn't exist
if ! getent group i2c > /dev/null 2>&1; then
    groupadd --system i2c
    log "Created i2c group"
fi

# Add udev rule for i2c devices if not present
UDEV_RULE="/etc/udev/rules.d/45-ddcutil-i2c.rules"
if [ ! -f "$UDEV_RULE" ]; then
    echo 'KERNEL=="i2c-[0-9]*", TAG+="uaccess"' > "$UDEV_RULE"
    log "Created udev rule for i2c access"
fi

# Reload udev rules
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger 2>/dev/null || true

# Add the installing user to i2c group
REAL_USER=${SUDO_USER:-$USER}
if [ -n "$REAL_USER" ] && [ "$REAL_USER" != "root" ]; then
    usermod -aG i2c "$REAL_USER"
    log "Added $REAL_USER to i2c group"
fi

# Update system caches
update-desktop-database -q /usr/share/applications 2>/dev/null || true
gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true

log "Installation completed successfully."
log "Note: Log out and back in for i2c permissions to take effect."

exit 0