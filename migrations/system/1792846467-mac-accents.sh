#!/usr/bin/env bash
# mac-accents: installs keyd, its config, the kubaos:mac-accents XKB option and
# the polkit rule behind the Mac-style accents on Left Alt, on a machine
# installed before system/50-hyprland.sh shipped them. Without them Left Alt is
# plain Alt and `kubaos keyboard accents` has no keyd to start.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_mac_accents
