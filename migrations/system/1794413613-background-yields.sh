#!/usr/bin/env bash
# background-yields: the power stops' root helper learns `yields=` and
# `unyield=`, which write nix-daemon's drop-in (Nix's builds at nice 19) and
# put the user's build tools back to nice 0 when background work stops
# yielding. This reinstalls the helper (install_power_helper in system/lib.sh;
# ppd's actions stay as the user set them); the power service writes the
# drop-in at its next stop, as background work yields by default.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_power_helper
