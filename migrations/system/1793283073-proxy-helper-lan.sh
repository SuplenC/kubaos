#!/usr/bin/env bash
# proxy-helper-lan: the Proxy's root helper again (install_proxy_helper in
# system/lib.sh), for the kill switch's `killswitch=lan|RANGES` form, which lets
# the network you're on through outside the tunnel. A helper installed before it
# rejects that form, so `kubaos-proxy` can't put the kill switch up with the
# local network allowed and keeps the rules it had.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_proxy_helper
