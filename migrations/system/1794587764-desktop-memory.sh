#!/usr/bin/env bash
# desktop-memory: the desktop's memory reserve (install_desktop_memory in
# system/lib.sh), on a machine installed before system/20-base.sh shipped
# it, so a build that fills RAM gives up its own pages before the
# compositor's, the shell's and the audio's. systemd-oomd arrives with the
# `memory` group's apt install, before it.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_desktop_memory
