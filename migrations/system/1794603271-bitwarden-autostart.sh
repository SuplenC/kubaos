#!/usr/bin/env bash
# bitwarden-autostart: Bitwarden desktop's system-wide autostart entry, on a
# machine installed before 70-bitwarden.sh shipped it. The session starts
# every XDG autostart entry, and Hyprland's config doesn't name Bitwarden, so
# without the entry its SSH agent would be down after login. The entry is the
# Apps catalog's Bitwarden's now (apps/bitwarden/), whose journal takes it in
# as KubaOS's when the app is picked; base-apps-setup takes it away again
# where Bitwarden isn't installed.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_root_file "$KUBAOS_DIR/apps/bitwarden/bitwarden.desktop" /etc/xdg/autostart/bitwarden.desktop 0644
