#!/usr/bin/env bash
# base-apps-setup: what Google Chrome, Docker and Bitwarden desktop need beyond
# their packages is their setup in the Apps catalog now (apps/<id>.toml's
# `setup`, run by its root helper), not the install's. On a machine with the
# app it is put in place again, from this release's files; on one without
# it, what the install put there for it goes: Chrome's theme helper and its
# units, Bitwarden's autostart entry, the docker group membership.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

# The setups are for the user who runs the update, whom sudo names.
[[ $TARGET_USER != root ]] || { echo "SUDO_USER doesn't name the user: run kubaos update as them" >&2; exit 1; }

if pkg_installed google-chrome-stable; then install_chrome_theme; else remove_chrome_theme; fi
if pkg_installed docker-ce; then install_docker_access; else remove_docker_access; fi
if pkg_installed bitwarden; then install_bitwarden_autostart; else remove_bitwarden_autostart; fi
