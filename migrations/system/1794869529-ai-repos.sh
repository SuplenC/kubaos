#!/usr/bin/env bash
# ai-repos: the apt sources of the catalog's AI apps (system/files/apt/sources.list.d/
# claude-desktop.sources, cursor.sources and windsurf.sources) and their keys put in place again
# where KubaOS set the repository up for the app, so a changed source reaches it. The guard is
# root's journal of the app, never the sources file: Cursor's own package writes the same file, so
# a Cursor installed by hand has it too, and that one stays the vendor's. A repository that can't
# be set up is a warning, not a failed update.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

repos=()
for pair in claude-desktop:claude-desktop cursor:cursor devin-desktop:windsurf; do
  if kubaos_app_set_up_repo "${pair%%:*}" "${pair#*:}"; then
    repos+=("${pair#*:}")
  fi
done
((${#repos[@]} == 0)) || repo_refresh "${repos[@]}"
