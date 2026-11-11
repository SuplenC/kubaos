#!/usr/bin/env bash
# default-palette: the boot screens in the default wallpaper's new palette, on a machine installed
# with the old one: the Plymouth splash's pictures and the GRUB theme's, drawn on the dark palette's
# background. The Plymouth theme is installed and then the initramfs rebuilt, since it carries its
# own copy of the splash for the disk prompt; the GRUB theme is files GRUB reads at boot, so
# grub.cfg stays as it is.
# kubaos update runs it once, as root, after apt and before the home-manager switch, under
# bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting non-zero stops the
# update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

# A machine without the theme never ran system/25-boot.sh, and keeps its boot as it is.
[[ -d $PLY_THEME_DIR ]] || exit 0
# GRUB's theme lives on /boot. Without /boot/grub, /boot isn't mounted, and the theme would land on
# the root filesystem, where GRUB can't read it.
[[ -d /boot/grub ]] || die "no /boot/grub: mount /boot and run kubaos update again"

# The rebuild runs whatever the install found: a rebuild that failed on an earlier run stopped the
# update with the files installed, and this run has to finish it. update-initramfs builds beside
# the image and moves it into place, so a failed one leaves the machine booting the old splash.
install_plymouth_theme
install_grub_theme
if in_container; then
  info "in a container: the initramfs stays as it is"
elif command -v update-initramfs >/dev/null; then
  sudo_run update-initramfs -u -k all
else
  warn "no update-initramfs here; the initramfs keeps the splash it has"
fi
