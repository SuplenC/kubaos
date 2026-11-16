#!/usr/bin/env bash
# browser-repos: the apt sources of the catalog's browsers (system/files/apt/sources.list.d/
# mozilla.sources, microsoft-edge.sources, vivaldi.sources and librewolf.sources) and their keys
# put in place again on a machine that has them, so a changed source reaches it. The Apps
# catalog's helper sets each repository up only as it installs its browser; a machine without
# one is left alone.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

for repo in mozilla microsoft-edge vivaldi librewolf; do
  if [[ -f /etc/apt/sources.list.d/$repo.sources ]]; then
    repo_setup "$repo"
  fi
done
