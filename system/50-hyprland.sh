#!/usr/bin/env bash
# Phase 5: Hyprland from ppa:cppiber/hyprland, held so apt never moves it,
# getty autologin on tty1, PAM services for the Quickshell lock screen, and
# keyd for the Mac-style accents on Left Alt.
#
# Hyprland itself is started by ~/.zprofile sourcing
# system/files/hyprland-autostart.zsh (exec start-hyprland on tty1).
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

PPA_URI="https://ppa.launchpadcontent.net/cppiber/hyprland/ubuntu"
PPA_KEY_FPR="A54D23B62FF3FCC76EFF71E8FDBAAA1CF0CCF48E"   # Launchpad PPA for Constantin Piber
PPA_KEY_URL="https://keyserver.ubuntu.com/pks/lookup?op=get&options=mr&search=0x$PPA_KEY_FPR"
PPA_KEYRING="/etc/apt/keyrings/cppiber-hyprland.gpg"

HYPR_PACKAGES=(hyprland hyprland-guiutils xdg-desktop-portal-hyprland hypridle hyprlock hyprpolkitagent hyprpicker)
AUTOLOGIN=1

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--user NAME] [--no-autologin]

  --user NAME      autologin user (default: $TARGET_USER)
  --no-autologin   remove the tty1 autologin drop-in instead of installing it
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --user) TARGET_USER="${2:?--user needs a name}"; shift ;;
    --no-autologin) AUTOLOGIN=0 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

require_sudo
apt_install ca-certificates curl gnupg

# --------------------------------------------------------------------------
step "Hyprland PPA (ppa:cppiber/hyprland, suite resolute)"
if is_amd64; then
  # deb822 file written directly: no software-properties-common needed.
  [[ -d /etc/apt/keyrings ]] || sudo_run install -d -m 0755 /etc/apt/keyrings
  fetch_key "$PPA_KEY_URL" "$PPA_KEYRING" "$PPA_KEY_FPR"
  add_apt_source cppiber-hyprland "Types: deb
URIs: $PPA_URI
Suites: resolute
Components: main
Signed-By: $PPA_KEYRING"
  apt_update_if_dirty
else
  # Checked 2026-10: the PPA builds only arch:all packages for arm64.
  warn "$(arch): the PPA has no $(arch) binaries. Using Ubuntu's own hyprland"
  warn "(0.53.x in resolute). It predates the Lua config (0.55+), so the"
  warn "session runs nixpkgs' Hyprland from home-manager instead (home/desktop.nix)."
  # hyprland-guiutils only exists in the PPA.
  pkgs=()
  for p in "${HYPR_PACKAGES[@]}"; do
    if pkg_has_candidate "$p"; then pkgs+=("$p"); else warn "no $p for $(arch), skipping it"; fi
  done
  HYPR_PACKAGES=("${pkgs[@]}")
fi

step "Install the Hyprland stack"
held=()
mapfile -t held < <(apt-mark showhold 2>/dev/null)
need_unhold=()
for p in "${HYPR_PACKAGES[@]}"; do
  if ! pkg_installed "$p" && printf '%s\n' "${held[@]}" | grep -qx "$p"; then need_unhold+=("$p"); fi
done
((${#need_unhold[@]})) && sudo_run apt-mark unhold "${need_unhold[@]}"
apt_install "${HYPR_PACKAGES[@]}"

step "Hold Hyprland and its hypr* libraries"
# Hold the top-level packages plus every installed hypr/aquamarine library, so
# a same-soname library bump can't slip in under a held compositor.
to_hold=("${HYPR_PACKAGES[@]}")
while IFS= read -r p; do
  to_hold+=("$p")
done < <(hypr_installed)
mapfile -t to_hold < <(printf '%s\n' "${to_hold[@]}" | sort -u)
mapfile -t held < <(apt-mark showhold 2>/dev/null)
missing_hold=()
for p in "${to_hold[@]}"; do
  pkg_installed "$p" || { ((DRY_RUN)) && missing_hold+=("$p"); continue; }
  printf '%s\n' "${held[@]}" | grep -qx "$p" || missing_hold+=("$p")
done
if ((${#missing_hold[@]})); then
  sudo_run apt-mark hold "${missing_hold[@]}"
else
  ok "already held: ${to_hold[*]}"
fi
if command -v Hyprland >/dev/null; then
  info "installed: $(Hyprland --version 2>/dev/null | head -1 || dpkg-query -W -f='${Version}' hyprland)"
fi
if command -v start-hyprland >/dev/null; then
  ok "start-hyprland present (the tty launcher)"
elif ((!DRY_RUN)); then
  warn "start-hyprland not found; edit system/files/hyprland-autostart.zsh to exec Hyprland"
fi

# --------------------------------------------------------------------------
step "Autologin on tty1 for $TARGET_USER"
DROPIN=/etc/systemd/system/getty@tty1.service.d/autologin.conf
if ((AUTOLOGIN)); then
  id "$TARGET_USER" >/dev/null 2>&1 || ((DRY_RUN)) || die "user $TARGET_USER does not exist"
  before="$(cat "$DROPIN" 2>/dev/null || true)"
  render_template "$FILES_DIR/getty/autologin.conf.in" "USER=$TARGET_USER" | write_root_file "$DROPIN" 0644
  if [[ "$before" != "$(cat "$DROPIN" 2>/dev/null || true)" ]] && has_systemd; then
    sudo_run systemctl daemon-reload
    info "active from the next boot (not restarting getty@tty1 under you)"
  fi
else
  if [[ -f "$DROPIN" ]]; then
    sudo_run rm -f "$DROPIN"
    has_systemd && sudo_run systemctl daemon-reload
  else
    ok "no autologin drop-in"
  fi
fi

# Hush login(1) on the autologin: no motd, no "Last login" between the boot
# splash and Hyprland. /etc/skel covers users made later (the install's
# useradd -m); the target user gets it now.
step "Quiet login (~/.hushlogin)"
if [[ -f /etc/skel/.hushlogin ]]; then
  ok "/etc/skel/.hushlogin present"
else
  sudo_run install -m 0644 /dev/null /etc/skel/.hushlogin
fi
if ((AUTOLOGIN)) && id "$TARGET_USER" >/dev/null 2>&1; then
  hush="$(user_home "$TARGET_USER")/.hushlogin"
  if [[ -e "$hush" ]]; then
    ok "$hush present"
  else
    sudo_run install -m 0644 -o "$TARGET_USER" -g "$(id -gn "$TARGET_USER")" /dev/null "$hush"
  fi
fi

# --------------------------------------------------------------------------
step "PAM services qs-lock + qs-lock-fingerprint (Quickshell lock screen)"
# Two services so password and fingerprint run in parallel (quickshell owner).
# Quickshell links Nix's libpam: see the comments in these files.
install_root_file "$FILES_DIR/pam.d/qs-lock" /etc/pam.d/qs-lock 0644
install_root_file "$FILES_DIR/pam.d/qs-lock-fingerprint" /etc/pam.d/qs-lock-fingerprint 0644
if [[ -e "$PAM_FPRINTD_LINK" ]]; then
  ok "$PAM_FPRINTD_LINK present"
else
  info "$PAM_FPRINTD_LINK (nixpkgs' module) is linked by system/60-nix.sh"
fi
# Nix's pam_unix.so runs /run/wrappers/bin/unix_chkpwd; link Ubuntu's helper there.
TMPFILES_CONF=/etc/tmpfiles.d/qs-lock-unix-chkpwd.conf
install_root_file "$FILES_DIR/tmpfiles.d/qs-lock-unix-chkpwd.conf" "$TMPFILES_CONF" 0644
if [[ "$(readlink /run/wrappers/bin/unix_chkpwd 2>/dev/null || true)" == /usr/sbin/unix_chkpwd ]]; then
  ok "/run/wrappers/bin/unix_chkpwd -> /usr/sbin/unix_chkpwd"
elif command -v systemd-tmpfiles >/dev/null; then
  sudo_run systemd-tmpfiles --create "$TMPFILES_CONF"
else
  sudo_run install -d -m 0755 /run/wrappers/bin
  sudo_run ln -sfn /usr/sbin/unix_chkpwd /run/wrappers/bin/unix_chkpwd
fi
if [[ -f /etc/pam.d/hyprlock ]]; then
  ok "/etc/pam.d/hyprlock present (fallback locker)"
else
  warn "/etc/pam.d/hyprlock missing; hyprlock may not be able to unlock. Create it with: auth include login"
fi

# --------------------------------------------------------------------------
step "Backlights writable by the video group (brightnessctl without root)"
install_root_file "$FILES_DIR/udev/90-kubaos-backlight.rules" /etc/udev/rules.d/90-kubaos-backlight.rules 0644
# No udev in an image build: the rule applies when the machine boots.
if udevadm control --ping 2>/dev/null; then
  sudo_run udevadm control --reload
  sudo_run udevadm trigger --subsystem-match=backlight --subsystem-match=leds --action=add
fi

# --------------------------------------------------------------------------
step "Mac-style accents on Left Alt (keyd, the kubaos:mac-accents XKB option)"
install_mac_accents

# --------------------------------------------------------------------------
step "Keyboard layouts changeable by an admin at the machine (polkit, pkexec)"
install_keyboard_writer

# --------------------------------------------------------------------------
step "Snapshot"
snap_create "hyprland base"

step "Done"
cat <<EOF
      ~/.zprofile must source the autostart snippet as its LAST line:
        source $FILES_DIR/hyprland-autostart.zsh
      (home-manager writes ~/.zprofile; see home/shell.nix.)
      Until then: log in on tty1 and run  start-hyprland
      Next: system/60-nix.sh
EOF
