#!/usr/bin/env bash
# graphics-repos: the apt sources of the graphics apps' vendor repositories
# (system/files/apt/sources.list.d/inkscape.sources and darktable.sources) and their keys put in
# place again where KubaOS set the repository up for the app, so a changed source reaches it:
# darktable's names Ubuntu's release, which moves on with it. The guard is root's journal of the
# app, never the sources file: a vendor's instructions write a source of the same repository by
# hand, and that one stays the user's. A repository that can't be set up is a warning, not a
# failed update.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

repos=()
for pair in inkscape:inkscape darktable:darktable; do
  if kubaos_app_set_up_repo "${pair%%:*}" "${pair#*:}"; then
    repos+=("${pair#*:}")
  fi
done
((${#repos[@]} == 0)) || repo_refresh "${repos[@]}"
