#!/usr/bin/env bash
# remove-workspace-merged: removes ~/.local/state/kubaos/hypr/workspace-merged,
# the state file of the workspaces that joined another screen on unplug, which
# nothing reads since workspace groups follow desk positions
# (config/hypr/workspaces.lua keeps workspace-screens instead).
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

rm -f -- "${XDG_STATE_HOME:-$HOME/.local/state}/kubaos/hypr/workspace-merged"
