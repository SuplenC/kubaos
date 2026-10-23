#!/usr/bin/env bash
# backlight-rule: installs the udev rule that makes the screen and keyboard
# backlights writable by the video group (system/files/udev), on a machine
# installed before system/50-hyprland.sh shipped it. Without it brightnessctl
# says "permission denied" and the brightness keys and slider do nothing.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

rule=/etc/udev/rules.d/90-kubaos-backlight.rules
src="$KUBAOS_DIR/system/files/udev/90-kubaos-backlight.rules"

if ! cmp -s "$src" "$rule"; then
  kubaos_backup "$rule"
  install -m 0644 "$src" "$rule"
  echo "installed $rule"
fi
udevadm control --reload
udevadm trigger --subsystem-match=backlight --subsystem-match=leds --action=add
