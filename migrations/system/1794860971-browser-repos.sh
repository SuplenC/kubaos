#!/usr/bin/env bash
# browser-repos: the apt sources of the catalog's browsers (system/files/apt/sources.list.d/
# mozilla.sources, microsoft-edge.sources, vivaldi.sources and librewolf.sources) and their keys
# put in place again where KubaOS set the repository up for the browser, so a changed source
# reaches it. The guard is root's journal of the browser, never the sources file: Edge's and
# Vivaldi's own packages write the same file, so a browser installed by hand has it too, and that
# one stays the vendor's. A repository that can't be set up is a warning, not a failed update.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

repos=()
for pair in firefox:mozilla microsoft-edge:microsoft-edge vivaldi:vivaldi librewolf:librewolf; do
  if kubaos_app_set_up_repo "${pair%%:*}" "${pair#*:}"; then
    repos+=("${pair#*:}")
  fi
done
((${#repos[@]} == 0)) || repo_refresh "${repos[@]}"
