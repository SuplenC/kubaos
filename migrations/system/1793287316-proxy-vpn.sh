#!/usr/bin/env bash
# proxy-vpn: the Proxy's root helper whose kill switch lets either VPN through
# (install_proxy_helper in system/lib.sh): WireGuard's wg-kubaos and its
# packets' mark, and OpenVPN's tun-kubaos with its servers by address and
# port. Without it the kill switch refuses an OpenVPN VPN's `server:` and
# `kubaos-proxy on` leaves the Proxy off. NetworkManager's OpenVPN plugin
# (network-manager-openvpn) comes from the manifest's `proxy` group, which the
# update installs.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_proxy_helper
