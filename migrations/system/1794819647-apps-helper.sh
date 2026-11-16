#!/usr/bin/env bash
# apps-helper: the Apps catalog's root helper and its polkit action
# (install_apps_helper in system/lib.sh), on a machine installed before
# 20-base.sh shipped them. `kubaos app add|remove` run the helper through
# pkexec and kubaos update through sudo; without it neither can install an app.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_apps_helper
