#!/usr/bin/env bash
# Points what system/ took from KubaOS's nixpkgs at the release now checked
# out: the wallpaper sets, the lock screen's PAM modules and the recorder's
# KMS helper (system/lib.sh). kubaos update and kubaos rollback run it after
# moving the release, before the home-manager switch: dot-theme reads the
# wallpaper link, and the helper must speak the new gsr's protocol. Each link
# is a no-op when it already points at the release's build.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

usage() {
  cat <<EOF
Usage: $0 [--dry-run]

Relinks the wallpaper sets, pam_fprintd.so, pam_u2f.so (when it is linked)
and the gsr-kms-server helper (when it is installed) to $KUBAOS_DIR.
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
if ! command -v nix >/dev/null && [[ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]]; then
  set +u
  # shellcheck disable=SC1091
  . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
  set -u
fi
command -v nix >/dev/null || ((DRY_RUN)) || die "nix not found: run system/60-nix.sh first"

step "Relinking to $KUBAOS_DIR"
link_wallpapers
link_pam_fprintd
if [[ -L "$PAM_U2F_LINK" ]] || ((DRY_RUN)); then link_pam_u2f; fi
if [[ -e /usr/local/libexec/dotfiles/gsr-kms-server ]]; then
  rec_args=()
  ((DRY_RUN)) && rec_args+=(--dry-run)
  run "$SYSTEM_DIR/65-recorder.sh" "${rec_args[@]}"
fi
