#!/usr/bin/env bash
# keyboard-writer: installs the root helper `kubaos keyboard` writes the default
# layout to /etc/default/keyboard with (for the console and the disk unlock), its
# polkit action, and the rule for that action in place of the old rule for
# localed, on a machine installed before system/50-hyprland.sh shipped them.
# Without them a new default never reaches the console: Ubuntu's localed, which
# `kubaos keyboard` asked before, turns SetX11Keyboard away with a D-Bus policy.
# The layouts themselves are the user's now (the keyboard-layouts user migration).
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

kubaos_backup /etc/polkit-1/rules.d/50-kubaos-keyboard.rules
install_keyboard_writer
