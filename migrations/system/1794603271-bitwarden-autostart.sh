#!/usr/bin/env bash
# bitwarden-autostart: Bitwarden desktop's system-wide autostart entry
# (install_bitwarden_autostart in system/lib.sh), on a machine installed before
# 70-bitwarden.sh shipped it. The session starts every XDG autostart entry,
# and Hyprland's config doesn't name Bitwarden, so without the entry its
# SSH agent would be down after login.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_bitwarden_autostart
