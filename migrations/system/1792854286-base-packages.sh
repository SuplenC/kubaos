#!/usr/bin/env bash
# base-packages: installs what system/20-base.sh's package list holds and this
# machine lacks. The list only runs at install time, so a package added to it
# later (Nautilus and its thumbnailers, viewers and archive tools) never
# reached a machine installed before. The list is read from the release's own
# 20-base.sh, so this installs exactly what a fresh install would.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

DRY_RUN=0
PACKAGES=()
# The array literal, as 20-base.sh writes it, and the line after it that adds
# the microcode on amd64.
eval "$(sed -n '/^PACKAGES=(/,/^)/p; /^is_amd64 && PACKAGES+=/p' "$KUBAOS_DIR/system/20-base.sh")"
((${#PACKAGES[@]})) || { echo "no package list in system/20-base.sh" >&2; exit 1; }

apt_install "${PACKAGES[@]}"
