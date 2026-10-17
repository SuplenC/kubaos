#!/usr/bin/env bash
# Phase 6: Nix (official multi-user installer), flakes, first home-manager
# switch for <user>@<host>, nixpkgs' pam_fprintd.so for the Quickshell lock,
# the one-time GPU setup for Nix apps, zsh as login shell.
#
# Installer choice: the official script from nixos.org (`--daemon`). It
# installs upstream Nix, the same Nix nixpkgs and home-manager are tested
# against, and it's what nixos.org recommends for Linux with systemd. The
# Determinate Systems installer stopped offering upstream Nix in early 2026
# and now installs Determinate Nix only (a vendor distribution with its own
# daemon and settings), which we don't need. Cost: no one-command uninstaller;
# see docs/INSTALL.md for the manual removal steps.
#
# Run as your normal user (not with sudo): the installer and home-manager must
# run as you; the script calls sudo itself where needed.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

# Pinned release of the official installer, checked against this hash before
# it runs (the script then checks the Nix tarball's own hash). To bump: pick a
# version from https://releases.nixos.org/?prefix=nix/ and copy its
# install.sha256.
NIX_VERSION="2.35.3"
NIX_INSTALL_URL="https://releases.nixos.org/nix/nix-$NIX_VERSION/install"
NIX_INSTALL_SHA256="69d0fe8fba8fe88f94c98b09964691ac9950abc7ecbf2ba861dc0533cfd901e9"
HM_USER="${USER:-$(id -un)}"
# Default host: this machine's hostname if the flake knows it, else tuxedo.
case "$(hostname -s 2>/dev/null || true)" in
  vm) HM_HOST="vm" ;;
  *) HM_HOST="tuxedo" ;;
esac
DO_SWITCH=1 DO_GPU=1 DO_CHSH=1 FORCE_GPU=0

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--user NAME] [--host NAME] [--no-switch] [--no-gpu] [--force-gpu] [--no-chsh]

  --user NAME   username part of the flake output (default: $HM_USER)
  --host NAME   host part of the flake output: tuxedo or vm (default: $HM_HOST)
                -> home-manager switch --flake $DOTFILES_DIR#<user>@<host>
  --no-switch   skip home-manager switch (the GPU step still runs if the
                current generation has non-nixos-gpu-setup)
  --no-gpu      skip the non-NixOS GPU setup
  --force-gpu   re-run the GPU setup even if it looks done
  --no-chsh     leave the login shell alone
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --user) HM_USER="${2:?--user needs a name}"; shift ;;
    --host) HM_HOST="${2:?--host needs a name}"; shift ;;
    --no-switch) DO_SWITCH=0 ;;
    --no-gpu) DO_GPU=0 ;;
    --force-gpu) FORCE_GPU=1 ;;
    --no-chsh) DO_CHSH=0 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

if ((EUID == 0)) && ! in_container; then
  die "run this as your normal user, not root (it calls sudo itself)"
fi
require_sudo
HM_CONFIG="$HM_USER@$HM_HOST"
if [[ "$(hostname -s 2>/dev/null || true)" != "$HM_HOST" ]] && ! in_container; then
  warn "hostname is $(hostname -s), not $HM_HOST: plain 'home-manager switch --flake $DOTFILES_DIR'"
  warn "won't find $HM_CONFIG. Fix with: sudo hostnamectl set-hostname $HM_HOST"
fi
NIX_BIN=/nix/var/nix/profiles/default/bin

# Make nix usable in this shell if it's installed but not on PATH yet.
load_nix_env() {
  if ! command -v nix >/dev/null && [[ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]]; then
    set +u
    # shellcheck disable=SC1091
    . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
    set -u
  fi
}

# --------------------------------------------------------------------------
step "Install Nix $NIX_VERSION (official installer, multi-user daemon)"
if [[ -x "$NIX_BIN/nix" ]]; then
  ok "Nix present: $("$NIX_BIN/nix" --version)"
else
  apt_install curl xz-utils
  if ((DRY_RUN)); then
    run curl --proto '=https' --tlsv1.2 -fsSL "$NIX_INSTALL_URL" -o "<tmp>/install"
    info "would check sha256 $NIX_INSTALL_SHA256, then:"
    run sh "<tmp>/install" --daemon --yes
  else
    tmp="$(mktemp -d)"
    curl --proto '=https' --tlsv1.2 -fsSL "$NIX_INSTALL_URL" -o "$tmp/install"
    if ! sha256sum --quiet -c <<<"$NIX_INSTALL_SHA256  $tmp/install"; then
      rm -rf "$tmp"
      die "Nix installer from $NIX_INSTALL_URL doesn't match the pinned sha256, not running it"
    fi
    run sh "$tmp/install" --daemon --yes
    rm -rf "$tmp"
  fi
fi
load_nix_env

step "Enable flakes in /etc/nix/nix.conf"
if [[ -f /etc/nix/nix.conf ]] && grep -Eq '^\s*experimental-features\s*=.*\bflakes\b' /etc/nix/nix.conf; then
  ok "flakes already enabled"
else
  if ((DRY_RUN)); then
    info "would append to /etc/nix/nix.conf: experimental-features = nix-command flakes"
  else
    sudo_run sh -c 'printf "\n# dotfiles/system/60-nix.sh\nexperimental-features = nix-command flakes\n" >> /etc/nix/nix.conf'
  fi
  if unit_exists nix-daemon.service && has_systemd; then
    sctl restart nix-daemon.service
  fi
fi
info "nix-daemon runs as a system service: tell IT, in case SentinelOne flags it."

# --------------------------------------------------------------------------
step "nix-daemon units as real files"
# The installer links /etc/systemd/system/nix-daemon.{socket,service} into
# /nix. /nix is its own subvolume (00-btrfs-subvolumes.md), and systemd reads
# units before it mounts it, so at boot the links dangle and the daemon never
# starts. Copies don't. A Nix upgrade that changes the units needs this again.
units_fixed=0
for u in nix-daemon.service nix-daemon.socket; do
  f="/etc/systemd/system/$u"
  if [[ -L "$f" && "$(readlink "$f")" == /nix/* ]]; then
    src="$(readlink -f "$f")"
    if [[ -f "$src" ]]; then
      sudo_run cp --remove-destination "$src" "$f"
      units_fixed=1
    else
      warn "$f points at $src, which isn't there"
    fi
  elif [[ -f "$f" ]]; then
    ok "$u is a real file"
  fi
done
if ((units_fixed)) && has_systemd; then
  sctl daemon-reload
fi

# --------------------------------------------------------------------------
if ((DO_SWITCH)); then
  step "home-manager switch --flake $DOTFILES_DIR#$HM_CONFIG"
  [[ -f "$DOTFILES_DIR/flake.nix" ]] || die "no flake.nix in $DOTFILES_DIR"
  # Flakes only see git-tracked files.
  if command -v git >/dev/null && [[ -n "$(git -C "$DOTFILES_DIR" ls-files --others --exclude-standard -- home config scripts flake.nix 2>/dev/null)" ]]; then
    warn "untracked files under home/ config/ scripts/ are invisible to the flake; git add them first"
  fi
  if command -v home-manager >/dev/null; then
    run home-manager switch -b hm-backup --flake "$DOTFILES_DIR#$HM_CONFIG"
  else
    # First run: use the home-manager the flake pins in flake.lock
    # (--inputs-from resolves "home-manager" to that input), so the CLI
    # matches the module version. Equivalent to
    #   nix run home-manager/release-26.05 -- switch --flake ...
    # (the flake pins release-26.05) without repeating the branch here.
    run nix run --inputs-from "$DOTFILES_DIR" home-manager -- \
      switch -b hm-backup --flake "$DOTFILES_DIR#$HM_CONFIG"
  fi
  info "-b hm-backup: files home-manager would overwrite are renamed *.hm-backup"
fi

# --------------------------------------------------------------------------
# /etc/pam.d/qs-lock-fingerprint (50-hyprland.sh) loads this link. Not tied to
# DO_SWITCH: `--no-switch --no-gpu --no-chsh` redoes only this step after a
# manual `nix flake update`.
step "pam_fprintd.so for the Quickshell lock ($PAM_FPRINTD_LINK)"
if ((DRY_RUN)) || command -v nix >/dev/null; then
  link_pam_fprintd
else
  warn "nix not found; skipping"
fi

# --------------------------------------------------------------------------
# setup_nix_gpu: home-manager's targets.genericLinux.gpu (home/desktop.nix)
# builds a driver bundle and ships `non-nixos-gpu-setup`. Run once as root; it
# writes /etc/tmpfiles.d/non-nixos-gpu.conf so /run/opengl-driver points at
# the bundle on every boot. home-manager warns on switch when it's stale
# ("GPU drivers require an update"), then run this again (--force-gpu).
setup_nix_gpu() {
  step "GPU drivers for Nix apps (/run/opengl-driver)"
  local setup=""
  for c in "$HOME/.nix-profile/bin/non-nixos-gpu-setup" \
    "${XDG_STATE_HOME:-$HOME/.local/state}/nix/profiles/home-manager/home-path/bin/non-nixos-gpu-setup" \
    "$(command -v non-nixos-gpu-setup 2>/dev/null || true)"; do
    [[ -n "$c" && -x "$c" ]] && { setup="$(readlink -f "$c")"; break; }
  done
  if [[ -z "$setup" ]]; then
    if ((DRY_RUN)); then
      sudo_run "<store path of non-nixos-gpu-setup>"
    elif ((!DO_SWITCH)); then
      info "non-nixos-gpu-setup not found (no home-manager generation yet?); skipping"
    else
      warn "non-nixos-gpu-setup not found; is targets.genericLinux.gpu.enable set? (home/desktop.nix)"
    fi
    return 0
  fi
  # The setup package carries the tmpfiles.d file it installs; if /etc has the
  # same content and the link exists, this generation is already set up.
  local pkg_conf
  pkg_conf="$(dirname "$(dirname "$setup")")/lib/tmpfiles.d/non-nixos-gpu.conf"
  if ((!FORCE_GPU)) && [[ -e /run/opengl-driver ]] &&
    cmp -s "$pkg_conf" /etc/tmpfiles.d/non-nixos-gpu.conf 2>/dev/null; then
    ok "/run/opengl-driver already set up by this generation"
    return 0
  fi
  # sudo with the absolute store path: root's PATH doesn't include ~/.nix-profile.
  sudo_run "$setup"
  info "test: nix run nixpkgs#mesa-demos -- eglinfo -B   (or start qs / ghostty)"
}

# Not tied to DO_SWITCH: `--no-switch --force-gpu` is the documented way to
# redo only this step after home-manager warns.
if ((DO_GPU)); then
  setup_nix_gpu
fi

# --------------------------------------------------------------------------
if ((DO_CHSH)); then
  step "Login shell: zsh (apt's /usr/bin/zsh; config from home-manager)"
  current="$(getent passwd "$HM_USER" | cut -d: -f7)"
  if [[ "$current" == /usr/bin/zsh || "$current" == /bin/zsh ]]; then
    ok "login shell is $current"
  elif [[ ! -x /usr/bin/zsh ]]; then
    warn "/usr/bin/zsh missing (system/20-base.sh installs it)"
  else
    sudo_run chsh -s /usr/bin/zsh "$HM_USER"
  fi
fi

step "Snapshot"
snap_create "home-manager base"

step "Done"
cat <<EOF
      Log out and back in (or reboot): tty1 autologin -> zsh -> start-hyprland.
      Outside Nix, on purpose (see docs/INSTALL.md):
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        curl -fsSL https://moonrepo.dev/install/proto.sh | bash
EOF
