#!/usr/bin/env bash
# zswap: compressed swap in RAM in front of the swap file (install_zswap in
# system/lib.sh), on a machine installed before system/20-base.sh shipped it,
# applied at once. Without it every swapped page is written to and read back
# from the encrypted disk, and a busy machine waits on it.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_zswap
