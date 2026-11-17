#!/usr/bin/env bash
# office-repos: the apt source of ONLYOFFICE's repository
# (system/files/apt/sources.list.d/onlyoffice.sources) and its key put in place again where KubaOS
# set the repository up for the app, so a changed source reaches it. The guard is root's journal
# of the app, never the sources file, which a repository the user set up by hand may share. A
# repository that can't be set up is a warning, not a failed update.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

if kubaos_app_set_up_repo onlyoffice onlyoffice; then
  repo_refresh onlyoffice
fi
