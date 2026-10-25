#!/usr/bin/env bash
# grouped-user-layer: moves each app's folder in the user layer from the flat
# layout, ~/.config/kubaos/<app>, into its group, ~/.config/kubaos/<group>/<app>
# (home/user-layer.nix). The switch before it has done the move already:
# `dot-config apply` relocates before it creates a missing file, so no seed
# lands where the user's file is about to go. This runs the same step, to be
# sure, and says what it could not move.
# kubaos update runs it once, as the user, after the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
#
# Nothing is lost: a folder moves whole when its group has none, else file by
# file, where an untouched seed gives way to the user's file and two files the
# user wrote are both kept (`<file>.old` beside it). A user layer that is a git
# clone sees renames; nothing is committed.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

"$KUBAOS_DIR/scripts/dot-config" relocate
