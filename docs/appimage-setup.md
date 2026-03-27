# AppImage setup

AppImage bundles the application into a single executable file but has no installer, so i2c access needs to be configured manually before running Sun.

## 1. Install ddcutil

```bash
# Debian, Ubuntu, Linux Mint
sudo apt install ddcutil

# Arch
sudo pacman -S ddcutil

# Fedora
sudo dnf install ddcutil
```

## 2. Configure i2c access

The recommended approach is to create a udev rule that grants your user access to i2c devices without needing sudo:

```bash
echo 'KERNEL=="i2c-[0-9]*", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/45-ddcutil-i2c.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Alternatively, add your user to the i2c group:

```bash
sudo usermod -aG i2c $USER
```

## 3. Log out and back in

This is required for the permissions to take effect in your session.

## 4. Verify ddcutil works

```bash
ddcutil detect
ddcutil getvcp 10
```

Both commands should work without sudo. If they do, Sun will work correctly.

## 5. Run Sun

```bash
chmod +x Sun_*.AppImage
./Sun_*.AppImage
```

## Troubleshooting

If `ddcutil detect` returns no displays, your monitor may not support DDC/CI or the feature may be disabled in the monitor's OSD settings. Look for a "DDC/CI" option in your monitor's menu and make sure it is enabled.