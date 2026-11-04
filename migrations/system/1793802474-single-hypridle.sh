#!/usr/bin/env bash
# single-hypridle: one hypridle in the session, the one `kubaos-idle session` starts, on a
# machine installed before system/50-hyprland.sh masked Ubuntu's hypridle.service for every user
# (retire_packaged_hypridle in system/lib.sh). With both running, every idle lock, screens-off and
# suspend fires twice. A packaged hypridle running now stops; the mask keeps it from the next login.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

retire_packaged_hypridle
