#!/usr/bin/env bash
# Phase 3b: the boot. Plymouth theme "kubaos" (the splash and the disk
# passphrase prompt), the GRUB theme, the menu named "KubaOS Shinjuku", a
# hidden menu that shows up when you need it. Runs after 20-base.sh (GRUB and
# grub-btrfs are there). Safe to re-run.
#
# What and why: docs/INSTALL-INTERNALS.md, "Boot". Assets: branding/boot.py.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

usage() {
  cat <<EOF
Usage: $0 [--dry-run]

Installs the KubaOS boot splash and GRUB theme, then rebuilds the initramfs
and grub.cfg (not in image builds: the install does that on the target).
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

require_sudo

# Not under /usr/share/kubaos: the payload build and the updater replace that
# checkout whole, and the wrapper fails without the script.
DIVERTED_10_LINUX=/usr/lib/kubaos/grub/10_linux.ubuntu

# --------------------------------------------------------------------------
step "Plymouth (script module, freetype labels; no pango, no extra fonts)"
group_install boot

step "Boot themes: Plymouth -> $PLY_THEME_DIR, GRUB -> $GRUB_THEME_DIR (on /boot: GRUB can't read the encrypted root)"
install_boot_theme

step "Plymouth on poweroff and reboot: once Hyprland has let go of the screen, and played out"
install_shutdown_splash

step "Snapshot boots: a throwaway overlay over the read-only snapshot"
install_root_file "$FILES_DIR/initramfs-tools/hooks/kubaos-snapshot-overlay" /etc/initramfs-tools/hooks/kubaos-snapshot-overlay 0755
install_root_file "$FILES_DIR/initramfs-tools/scripts/local-bottom/kubaos-snapshot-overlay" \
  /etc/initramfs-tools/scripts/local-bottom/kubaos-snapshot-overlay 0755
if unit_exists snapper-boot.service || [[ -f /usr/lib/systemd/system/snapper-boot.service ]]; then
  install_root_file "$FILES_DIR/systemd/snapper-boot.service.d/kubaos-not-in-snapshot.conf" \
    /etc/systemd/system/snapper-boot.service.d/kubaos-not-in-snapshot.conf 0644
fi

# Ubuntu's initramfs hook picks the theme from this alternative, plymouthd
# from plymouthd.conf: set both.
default_alt=/usr/share/plymouth/themes/default.plymouth
theme_file="$PLY_THEME_DIR/kubaos.plymouth"
if [[ "$(update-alternatives --query default.plymouth 2>/dev/null | sed -n 's/^Value: //p')" == "$theme_file" ]]; then
  ok "default.plymouth -> kubaos"
else
  sudo_run update-alternatives --install "$default_alt" default.plymouth "$theme_file" 200
  sudo_run update-alternatives --set default.plymouth "$theme_file"
fi

# --------------------------------------------------------------------------
step "GRUB: entries named after os-release (10_linux wrapper, packaged script diverted)"
# 10_linux appends "GNU/Linux" to any distributor but Ubuntu. Divert Ubuntu's
# script out of /etc/grub.d (grub-mkconfig runs everything executable there)
# and put a wrapper in its place; grub upgrades update the diverted copy.
divert="$(dpkg-divert --truename /etc/grub.d/10_linux)"
if [[ "$divert" == "$DIVERTED_10_LINUX" ]]; then
  ok "/etc/grub.d/10_linux diverted to $DIVERTED_10_LINUX"
elif [[ "$divert" != /etc/grub.d/10_linux ]]; then
  # Diverted to the old place, inside the checkout: move it out.
  sudo_run install -d -m 0755 "$(dirname "$DIVERTED_10_LINUX")"
  sudo_run dpkg-divert --local --no-rename --remove /etc/grub.d/10_linux
  sudo_run dpkg-divert --local --no-rename --divert "$DIVERTED_10_LINUX" --add /etc/grub.d/10_linux
  if [[ -f "$divert" ]]; then
    sudo_run mv "$divert" "$DIVERTED_10_LINUX"
  else
    warn "$divert is gone; reinstall grub2-common to get Ubuntu's 10_linux back at $DIVERTED_10_LINUX"
  fi
elif [[ -f /etc/grub.d/10_linux ]] || ((DRY_RUN)); then
  sudo_run install -d -m 0755 "$(dirname "$DIVERTED_10_LINUX")"
  sudo_run dpkg-divert --local --rename --divert "$DIVERTED_10_LINUX" --add /etc/grub.d/10_linux
else
  die "no /etc/grub.d/10_linux (is grub2-common installed?)"
fi
install_root_file "$FILES_DIR/grub/10_linux" /etc/grub.d/10_linux 0755
install_root_file "$FILES_DIR/grub/kubaos.cfg" /etc/default/grub.d/10-kubaos.cfg 0644

# grub-btrfs titles each snapshot "| date | @snapshots/N/snapshot | type |
# description |", too wide for the menu. Date and what it was is enough.
btrfs_cfg=/etc/default/grub-btrfs/config
title='GRUB_BTRFS_TITLE_FORMAT=("date" "description")'
if [[ ! -f "$btrfs_cfg" ]]; then
  info "no $btrfs_cfg (grub-btrfs comes with 20-base.sh); snapshot titles left alone"
elif grep -qxF "$title" "$btrfs_cfg"; then
  ok "snapshot entries titled by date and description"
else
  sudo_run sed -i -e '/^GRUB_BTRFS_TITLE_FORMAT=/d' -e "\$a $title" "$btrfs_cfg"
fi

# --------------------------------------------------------------------------
if ((IMAGE_BUILD)); then
  step "Image build: the install rebuilds the initramfs and grub.cfg on the target"
elif ! command -v update-grub >/dev/null; then
  warn "no update-grub here; skipping the initramfs and grub.cfg"
else
  step "Rebuild the initramfs (splash and theme in it) and grub.cfg"
  sudo_run update-initramfs -u -k all
  sudo_run update-grub
fi

step "Done"
info "Check: update-alternatives --query default.plymouth"
info "       lsinitramfs /boot/initrd.img-\$(uname -r) | grep -E 'kubaos|Plymouth.ttf|label-freetype'"
info "       grep -E \"^(menuentry|submenu) \" /boot/grub/grub.cfg"
