#!/usr/bin/env bash
# network-failover: a cable and Wi-Fi on one network both stay up, the cable
# carrying the traffic while it reaches the internet, on a machine installed
# before system/20-base.sh shipped it. Each interface answers ARP only for its
# own addresses (install_arp_per_interface), so the Wi-Fi card no longer claims
# the cable's DHCP lease and NetworkManager stops dropping it as a conflict; and
# NetworkManager's internet check is on (install_connectivity_check), so a link
# with no internet behind it loses the default route to the other. Both apply at
# once, without dropping a connection.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_arp_per_interface
install_connectivity_check
