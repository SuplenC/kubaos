#!/usr/bin/env bash
# apps-helper-binary: the Apps catalog's root helper is the kubaos-apps
# binary of the release's build now, which runs the apps' typed actions,
# not the shell script that installed packages and ran system hooks. This
# takes the script away and reinstalls the polkit action pkexec runs the
# helper under (install_apps_policy in system/lib.sh, polkit/org.kubaos.apps.policy);
# system/relink.sh, which every update runs once the system migrations are
# done, links the binary in its place (install_apps_helper).
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

# The script, never a link: once relink.sh has run, the link stays.
if [[ -f $APPS_LINK && ! -L $APPS_LINK ]]; then
  sudo_run rm -f "$APPS_LINK"
fi
install_apps_policy
