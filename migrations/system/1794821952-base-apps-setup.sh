#!/usr/bin/env bash
# base-apps-setup: what Google Chrome and Bitwarden desktop had on the system
# beyond their packages, for KubaOS alone, goes from a machine without the app:
# Bitwarden's autostart entry, and Chrome's theme helper, its units and the
# policy it wrote, unless another Chrome build (beta, unstable) is there and
# reads them. On a machine with the app they are the Apps catalog's app's
# steps now (apps/<id>/app.toml), which take them in when the app is picked
# (`kubaos app adopt`, the user migration apps-adopt) and bring them to this
# release's on the next update: their files moved from system/files/
# (chrome/dot-chrome-theme, chrome/dot-chrome-theme.path.in,
# chrome/dot-chrome-theme.service.in, xdg/autostart/bitwarden.desktop) into
# the apps' folders. Docker's group and drop-in stay without
# docker-ce: a Docker engine of the user's own (docker.io) has them too, and
# the drop-in comes with the base (docker_on_demand) for any of them.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

if ! chrome_installed; then
  remove_chrome_theme
fi
if ! pkg_installed bitwarden; then
  remove_bitwarden_autostart
fi
