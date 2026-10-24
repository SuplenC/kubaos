#!/usr/bin/env bash
# keyboard-rule: installs the polkit rule that lets an admin at the machine
# change the keyboard layouts without a password (system/files/polkit), on a
# machine installed before system/50-hyprland.sh shipped it. Without it a
# change from the control center's Keyboard tab can fail with "Access denied"
# and no password prompt.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

rule=/etc/polkit-1/rules.d/50-kubaos-keyboard.rules
src="$KUBAOS_DIR/system/files/polkit/50-kubaos-keyboard.rules"

# polkitd watches rules.d and reads a new rule on its own.
if ! cmp -s "$src" "$rule"; then
  kubaos_backup "$rule"
  install -D -m 0644 "$src" "$rule"
  echo "installed $rule"
fi
