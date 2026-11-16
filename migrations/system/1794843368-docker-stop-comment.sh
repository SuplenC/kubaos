#!/usr/bin/env bash
# docker-stop-comment: Docker's stop timeout drop-in names what installs it
# now, Docker's app and docker_on_demand (system/lib.sh), in its comment. This
# writes the new text where a Docker has the drop-in, as docker_on_demand
# installs it; the setting itself is the same.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

dropin=/etc/systemd/system/docker.service.d/kubaos-stop.conf
[[ -f $dropin ]] || exit 0
install_root_file "$FILES_DIR/systemd/docker.service.d/kubaos-stop.conf" "$dropin" 0644
