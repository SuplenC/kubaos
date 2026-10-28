#!/usr/bin/env bash
# boot-theme-tokens: the boot screens drawn from the design tokens, on a machine installed before
# them: the passphrase field as the lock screen's, its keycaps and notes, the GRUB menu's rows and
# key hints, and Geist Mono as the boot's monospace. Each theme is installed and then what boots
# from it rebuilt: the Plymouth theme and the initramfs, which carries its own copy of the splash
# for the disk prompt, then the GRUB theme and grub.cfg, which loads the theme's fonts by file.
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

# The rebuilds run whatever the installs found: a rebuild that failed on an earlier run stopped
# the update with the files installed, and this run has to finish it. A rebuild that fails
# leaves the machine booting the old file: update-initramfs builds beside the image and moves it
# into place, and grub-mkconfig writes grub.cfg.new and checks it before copying it over grub.cfg.
install_plymouth_theme
if in_container; then
  install_grub_theme
  info "in a container: the initramfs and grub.cfg stay as they are"
  exit 0
fi
if command -v update-initramfs >/dev/null; then
  sudo_run update-initramfs -u -k all
else
  warn "no update-initramfs here; the initramfs keeps the splash it has"
fi
install_grub_theme
if command -v update-grub >/dev/null; then
  sudo_run update-grub
else
  warn "no update-grub here; grub.cfg stays as it is"
fi
