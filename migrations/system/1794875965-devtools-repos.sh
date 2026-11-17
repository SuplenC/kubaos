#!/usr/bin/env bash
# devtools-repos: the apt sources of the developer tools' vendor repositories
# (system/files/apt/sources.list.d/dbeaver.sources, bruno.sources, lens.sources, kubernetes.sources
# and winehq.sources) and their keys put in place again where KubaOS set the repository up for
# the app, so a changed source reaches it: kubectl's names a minor version of Kubernetes, which a
# release moves on. The guard is root's journal of the app, never the sources file: a vendor's
# instructions write a source of the same repository by hand, and that one stays the user's. A
# repository that can't be set up is a warning, not a failed update.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

repos=()
for pair in dbeaver:dbeaver bruno:bruno lens:lens kubectl:kubernetes wine:winehq; do
  if kubaos_app_set_up_repo "${pair%%:*}" "${pair#*:}"; then
    repos+=("${pair#*:}")
  fi
done
((${#repos[@]} == 0)) || repo_refresh "${repos[@]}"
