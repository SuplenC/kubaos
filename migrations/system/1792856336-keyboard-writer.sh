#!/usr/bin/env bash
# keyboard-writer: installs the root helper `kubaos keyboard` writes
# /etc/default/keyboard with, its polkit action and the rule for that action, on
# a machine installed before system/50-hyprland.sh shipped them. Without them
# every change to the keyboard layouts is refused: Ubuntu's localed, which
# `kubaos keyboard` asked before, turns SetX11Keyboard away with a D-Bus policy.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

kubaos_backup /etc/polkit-1/rules.d/50-kubaos-keyboard.rules
install_keyboard_writer
