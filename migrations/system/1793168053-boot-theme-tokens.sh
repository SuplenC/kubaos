#!/usr/bin/env bash
# boot-theme-tokens: the boot screens drawn from the design tokens
# (install_boot_theme in system/lib.sh), on a machine installed before them:
# the passphrase field as the lock screen's, its keycaps and notes, the GRUB
# menu's rows and key hints, and Geist Mono as the boot's monospace. Then the
# initramfs, which carries its own copy of the splash for the disk prompt, and
# grub.cfg, which loads the theme's fonts by name, are rebuilt.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

# A machine without the theme never ran system/25-boot.sh, and keeps its boot
# as it is.
[[ -d $PLY_THEME_DIR ]] || exit 0

install_boot_theme

if in_container; then
  info "in a container: the initramfs and grub.cfg stay as they are"
  exit 0
fi
# Rebuilt whatever install_boot_theme found: a rebuild that failed on an
# earlier run, which stopped the update, left the files installed and the
# initramfs as it was, and this run has to finish it. update-initramfs
# replaces an image only once the new one is built.
if command -v update-initramfs >/dev/null; then
  sudo_run update-initramfs -u -k all
fi
if command -v update-grub >/dev/null; then
  sudo_run update-grub
fi
