#!/usr/bin/env bash
# boot-sign-layers: the boot splash's switch-on layers drawn again, on a machine installed with the
# old ones: a lit group's glow now carries on past its dark neighbours, and the layers no longer
# carry the splash's background. The Plymouth theme is installed and then the initramfs rebuilt,
# since it carries its own copy of the splash for the disk prompt.
# kubaos update runs it once, as root, after apt and before the home-manager switch, under
# bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting non-zero stops the
# update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

# A machine without the theme never ran system/25-boot.sh, and keeps its boot as it is.
[[ -d $PLY_THEME_DIR ]] || exit 0

# The rebuild runs whatever the install found: a rebuild that failed on an earlier run stopped the
# update with the files installed, and this run has to finish it. update-initramfs builds beside
# the image and moves it into place, so a failed one leaves the machine booting the old splash.
install_plymouth_theme
if in_container; then
  info "in a container: the initramfs stays as it is"
elif command -v update-initramfs >/dev/null; then
  sudo_run update-initramfs -u -k all
else
  warn "no update-initramfs here; the initramfs keeps the splash it has"
fi
