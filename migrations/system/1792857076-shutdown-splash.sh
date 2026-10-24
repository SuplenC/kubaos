#!/usr/bin/env bash
# shutdown-splash: Plymouth's poweroff and reboot units ordered after the
# login sessions, and the unit that lets the splash play out before the
# machine goes off (install_shutdown_splash in system/lib.sh), on a machine
# installed before system/25-boot.sh did this. Without them plymouthd mostly
# starts while Hyprland still holds the screen, and the sign never shows.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_shutdown_splash
