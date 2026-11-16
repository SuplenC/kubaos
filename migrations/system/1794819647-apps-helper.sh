#!/usr/bin/env bash
# apps-helper: the polkit action the Apps catalog's root helper runs under
# (install_apps_policy in system/lib.sh), on a machine installed before the
# catalog. `kubaos app add|remove` run the helper through pkexec; without the
# action, pkexec refuses it. The helper itself is linked by system/relink.sh,
# which every update runs once the system migrations are done.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_apps_policy
