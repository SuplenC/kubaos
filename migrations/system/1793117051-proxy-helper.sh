#!/usr/bin/env bash
# proxy-helper: what the Proxy needs as root (install_proxy_helper in
# system/lib.sh), on a machine installed before system/20-base.sh shipped it:
# the helper that puts the work network's system-wide parts in place and takes
# them away, the same script as the pin helper, their polkit actions and the
# rule. Without them `kubaos-proxy on`
# stops at those parts and leaves the Proxy off. Its packages (wireguard-tools,
# nftables, openssl) come from the manifest's `proxy` group, which the update
# installs.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_proxy_helper
