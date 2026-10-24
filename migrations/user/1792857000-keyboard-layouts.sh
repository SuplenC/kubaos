#!/usr/bin/env bash
# keyboard-layouts: keeps the layouts in /etc/default/keyboard as the user's own
# list, `layouts` in ~/.config/kubaos/keyboard/keyboard.toml, which Hyprland
# reads and `kubaos keyboard` changes from now on without asking for root. A
# user who has a list already keeps it. Until this runs Hyprland reads the same
# layouts from /etc/default/keyboard, so nothing changes under the user's hands.
# kubaos update runs it once, as the user, after the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

settings="${XDG_CONFIG_HOME:-$HOME/.config}/kubaos/keyboard/keyboard.toml"
kubaos_backup "$settings"
kubaos keyboard seed
