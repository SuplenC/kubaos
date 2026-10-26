#!/usr/bin/env bash
# binary-cache: KubaOS's binary cache (kuba-os.cachix.org) in /etc/nix/nix.conf
# (nix_binary_cache in system/lib.sh), on a machine installed before
# system/60-nix.sh wrote it. Without it the switch after a release builds
# KubaOS's crates and Kotodama on the machine, where CI has built them
# already. The file's other lines stay, and so do lines naming the cache or
# its key already.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

kubaos_backup /etc/nix/nix.conf
nix_binary_cache
