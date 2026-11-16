#!/usr/bin/env bash
# apps-catalog-handover: Google Chrome's and Docker's packages are the Apps
# catalog's (apps/google-chrome/, apps/docker/), so they leave the
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

# The packages the docker and chrome groups list, which the catalog's apps
# install, and those apps' as this release has them: a group may be gone
# from a later release, and the record loses whatever of them it has.
mapfile -t packages < <(
  manifest_value group.docker packages
  manifest_value group.chrome packages
  printf '%s\n' docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin google-chrome-stable
)
((${#packages[@]} == 0)) || record_drop "${packages[@]}"
