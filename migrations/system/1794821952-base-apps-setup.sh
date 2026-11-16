#!/usr/bin/env bash
# base-apps-setup: what Google Chrome, Docker and Bitwarden desktop need on
# the system beyond their packages is their system hooks in the Apps catalog
# now (apps/<id>.toml's `system_hooks`, run by its root helper), not the
# install's. Each app the user migration base-apps-picked records as picked
# (installed, and its package group not turned off) gets its hooks again,
# from this release's files, Docker's shutdown drop-in (kubaos-stop.conf,
# from docker_on_demand) among them. One installed while its group was off
# was the user's own, set up by hand, and is left as it is.
# Of a machine without the app, only what the install put there for KubaOS
# alone goes: Bitwarden's autostart entry, and Chrome's theme helper and its
# units unless another Chrome build (beta, unstable) reads them. The docker
# group membership and the drop-in stay without docker-ce: a Docker engine of
# the user's own (docker.io) has them too, and the drop-in comes with the
# base (install_fast_shutdown_boot) for any of them.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

# The hooks are for the user who runs the update, whom sudo names.
[[ $TARGET_USER != root ]] || { echo "SUDO_USER doesn't name the user: run kubaos update as them" >&2; exit 1; }

# picked PACKAGE GROUP: whether base-apps-picked picks the app PACKAGE says is
# there: it is installed, and the user didn't turn GROUP off.
picked() {
  pkg_installed "$1" && [[ "$(group_choice "$2")" != off ]]
}

if picked google-chrome-stable chrome; then
  install_chrome_theme
elif ! chrome_installed; then
  remove_chrome_theme
fi
if picked docker-ce docker; then
  install_docker_access
fi
if pkg_installed bitwarden; then
  install_bitwarden_autostart
else
  remove_bitwarden_autostart
fi
