#!/usr/bin/env bash
# parallel-builds: `max-jobs = auto` in /etc/nix/nix.conf (nix_parallel_builds
# in system/lib.sh), on a machine installed before system/60-nix.sh wrote it.
# Without it the daemon builds one derivation at a time, and the switch after
# a release builds KubaOS's crates one after another. The file's other lines
# stay, and so does a max-jobs line already there.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

kubaos_backup /etc/nix/nix.conf
nix_parallel_builds
