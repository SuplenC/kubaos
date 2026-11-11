#!/usr/bin/env bash
# project-pins-on-mise: moves the `use suite` and `use bwenv` lines of every
# .envrc under the Projects root to the folder's mise.toml, since mise is the
# shell's directory hook and direnv no longer runs. A pin becomes the Suite's
# main tool in [tools]; `use bwenv` becomes KUBAOS_BWENV and a `_.source` of
# ~/.local/share/kubaos/bwenv.sh in [env], trusted in mise. A line it can't
# move (a Suite not on mise, a line in a block) stays, and `kubaos pins`
# names it. Each file it changes is backed up under KUBAOS_BACKUPS first, and
# an .envrc left with nothing else is deleted (crates/kubaos-core/src/pins.rs,
# `Pins::migrate`).
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

kubaos pins --migrate
