#!/usr/bin/env bash
# sleep-safe-power: systemd-sleep's hook for the power stops, and the root
# helper's --sleep-safe it runs (install_power_helper in system/lib.sh), on a
# machine installed before they shipped. Without them a suspend from
# Performance or Max Performance keeps ASPM, USB and the GPU awake, never
# reaches s0i3, and drains the battery while asleep. ppd's actions stay as the
# user set them: no --actions.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_power_helper
