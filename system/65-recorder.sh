#!/usr/bin/env bash
# Phase 6b: KMS capture for gpu-screen-recorder (scripts/dot-hypr-record).
#
# gsr (Nix, home/desktop.nix) records a monitor by asking its small helper
# gsr-kms-server for the framebuffer, which needs CAP_SYS_ADMIN. A file
# capability can't live in the read-only Nix store, so this script:
#   - builds nixpkgs#gpu-screen-recorder from KubaOS's pinned nixpkgs
#     (KUBAOS_DIR, system/lib.sh: the same store path home-manager installs
#     from the curated set) and pins it with a GC root, so
#     the helper's libraries stay;
#   - copies gsr-kms-server to /usr/local/libexec/dotfiles/gsr-kms-server,
#     root:<you> 0750 with cap_sys_admin+ep: only your user can run it;
#   - links /run/wrappers/bin/gsr-kms-server to it (tmpfiles.d). nixpkgs' gsr
#     wrapper puts /run/wrappers/bin first on its PATH to find the helper,
#     the NixOS module's place for it.
# Without this, gsr falls back to pkexec and asks for the password each time.
# Re-run after every KubaOS update (system/update.sh --nix does): the helper
# and gsr must speak the same protocol version.
#
# Why not the portal (-w portal, no privileges): gsr's portal capture is
# broken on Hyprland until xdg-desktop-portal-hyprland ships commit ba31964
# (2026-08-29, after v1.4.1), it shows the portal's picker instead of slurp,
# and it can't crop to a region. Switch to it when a fixed xdph is in the PPA.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--remove]

  --remove   delete the helper copy, its link and GC root (gsr then asks for the password)
$(common_usage)
EOF
}

REMOVE=0
while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --remove) REMOVE=1 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

HELPER=/usr/local/libexec/dotfiles/gsr-kms-server
WRAPPER=/run/wrappers/bin/gsr-kms-server
GCROOT=/nix/var/nix/gcroots/dotfiles-gpu-screen-recorder
TMPFILES=/etc/tmpfiles.d/dotfiles-gsr.conf

require_sudo

if ((REMOVE)); then
  step "Removing the gsr-kms-server helper"
  sudo_run rm -f "$WRAPPER" "$HELPER" "$TMPFILES" "$GCROOT"
  exit 0
fi

step "Packages"
apt_install libcap2-bin # setcap, getcap

step "gsr-kms-server with CAP_SYS_ADMIN"
command -v nix >/dev/null || ((DRY_RUN)) || die "nix not found: run system/60-nix.sh first"
if ((DRY_RUN)); then
  run nix build --no-link --print-out-paths --inputs-from "$KUBAOS_DIR" nixpkgs#gpu-screen-recorder
  out="<gpu-screen-recorder store path>"
else
  out="$(nix build --no-link --print-out-paths --inputs-from "$KUBAOS_DIR" nixpkgs#gpu-screen-recorder)" ||
    die "nix build nixpkgs#gpu-screen-recorder failed (the error is above); fix that, then re-run"
  [[ -x "$out/bin/gsr-kms-server" ]] || die "no gsr-kms-server in $out"
fi
group="$(id -gn "$TARGET_USER")"
if ((!DRY_RUN)) && [[ "$(readlink "$GCROOT" 2>/dev/null)" == "$out" ]] &&
  cmp -s "$out/bin/gsr-kms-server" "$HELPER" &&
  [[ "$(stat -c '%U:%G %a' "$HELPER")" == "root:$group 750" ]] &&
  getcap "$HELPER" | grep -q 'cap_sys_admin[=+]ep'; then
  ok "$HELPER is current ($out)"
else
  # GC root first, so the libraries the copy links against can't be collected.
  sudo_run ln -sfn "$out" "$GCROOT"
  sudo_run install -D -o root -g "$group" -m 0750 "$out/bin/gsr-kms-server" "$HELPER"
  sudo_run setcap cap_sys_admin+ep "$HELPER"
fi

step "Link in /run/wrappers/bin"
write_root_file "$TMPFILES" 0644 <<EOF
# Written by dotfiles/system/65-recorder.sh: gpu-screen-recorder finds its
# setcap'd KMS helper here (nixpkgs' wrapper puts /run/wrappers/bin on PATH).
d /run/wrappers     0755 root root -
d /run/wrappers/bin 0755 root root -
L+ $WRAPPER - - - - $HELPER
EOF
if [[ "$(readlink "$WRAPPER" 2>/dev/null)" == "$HELPER" ]]; then
  ok "$WRAPPER -> $HELPER"
elif has_systemd; then
  sudo_run systemd-tmpfiles --create "$TMPFILES"
else
  sudo_run install -d -m 0755 /run/wrappers/bin
  sudo_run ln -sfn "$HELPER" "$WRAPPER"
fi
ok "dot-hypr-record can capture monitors without a password prompt"
