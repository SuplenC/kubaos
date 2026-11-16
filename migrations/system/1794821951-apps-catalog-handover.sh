#!/usr/bin/env bash
# apps-catalog-handover: Google Chrome's and Docker's packages are the Apps
# catalog's (apps/google-chrome.toml, apps/docker.toml), so they leave the
# record of what the package manifest installed. kubaos leaves the catalog's
# packages alone, and an app goes only through `kubaos app remove`; out of the
# record, no update's sync removes them either. Nothing is installed or
# removed. The user migration base-apps-picked records them as picked.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

# The packages the docker and chrome groups list, and the catalog's apps
# install: either may be gone from a later release, and the record loses
# whatever of them it has.
mapfile -t packages < <(
  manifest_value group.docker packages
  manifest_value group.chrome packages
  for app in docker google-chrome; do
    [[ ! -f $KUBAOS_DIR/apps/$app.toml ]] ||
      manifest_dump "$KUBAOS_DIR/apps/$app.toml" | awk -F'\t' '$1 == "apt" && $2 == "packages" { print $3 }'
  done
)
((${#packages[@]} == 0)) || record_drop "${packages[@]}"
