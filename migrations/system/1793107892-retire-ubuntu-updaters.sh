#!/usr/bin/env bash
# retire-ubuntu-updaters: Ubuntu's own updaters off (retire_ubuntu_updaters in system/lib.sh) on a
# machine installed before system/10-strip.sh retired them: unattended-upgrades and the apt-daily
# timers stopped and masked, and apt's periodic jobs set to 0. Left on, they run apt in the
# background, where they can hold the dpkg lock while `kubaos update` runs.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

retire_ubuntu_updaters
