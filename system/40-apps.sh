#!/usr/bin/env bash
# Phase 4 (apps): Docker CE from Docker's repo, Google Chrome from Google's
# repo.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

DOCKER_KEY_URL="https://download.docker.com/linux/ubuntu/gpg"
DOCKER_KEY_FPR="9DC858229FC7DD38854AE2D88D81803C0EBFCD88"
DOCKER_KEYRING="/etc/apt/keyrings/docker.gpg"
# Chrome's own postinst writes google-chrome.sources with exactly this key
# path and URI; using the same values means it won't fight our file.
CHROME_KEY_URL="https://dl.google.com/linux/linux_signing_key.pub"
CHROME_KEY_FPR="EB4C1BFD4F042F6DDDCCEC917721F63BD38B4796"
CHROME_KEYRING="/usr/share/keyrings/google-chrome.gpg"
DO_DOCKER=1 DO_CHROME=1

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--no-docker] [--no-chrome]

  --no-docker     skip Docker
  --no-chrome     skip Chrome
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --no-docker) DO_DOCKER=0 ;;
    --no-chrome) DO_CHROME=0 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

require_sudo
apt_install ca-certificates curl gnupg
[[ -d /etc/apt/keyrings ]] || sudo_run install -d -m 0755 /etc/apt/keyrings

# --------------------------------------------------------------------------
if ((DO_DOCKER)); then
  step "Docker CE from download.docker.com (suite resolute)"
  # Docker's install guide: remove distro packages that conflict.
  apt_purge docker.io docker-compose docker-compose-v2 docker-doc podman-docker containerd runc
  fetch_key "$DOCKER_KEY_URL" "$DOCKER_KEYRING" "$DOCKER_KEY_FPR"
  add_apt_source docker "Types: deb
URIs: https://download.docker.com/linux/ubuntu
Suites: resolute
Components: stable
Architectures: $(arch)
Signed-By: $DOCKER_KEYRING"
  apt_update_if_dirty
  apt_install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

  if id -nG "$TARGET_USER" 2>/dev/null | tr ' ' '\n' | grep -qx docker; then
    ok "$TARGET_USER is in the docker group"
  else
    getent group docker >/dev/null || sudo_run groupadd docker
    sudo_run usermod -aG docker "$TARGET_USER"
    info "log out and back in (or reboot) for the docker group to apply"
  fi
  docker_on_demand
  if findmnt -n /var/lib/docker >/dev/null 2>&1; then
    ok "/var/lib/docker is its own mount (kept out of root snapshots)"
  else
    info "/var/lib/docker lives inside @ and will be in every root snapshot (see 00-btrfs-subvolumes.md, @docker)"
  fi
fi

# --------------------------------------------------------------------------
if ((DO_CHROME)); then
  step "Google Chrome"
  if ! is_amd64; then
    warn "Chrome is amd64 only; skipping on $(arch)"
  else
    fetch_key "$CHROME_KEY_URL" "$CHROME_KEYRING" "$CHROME_KEY_FPR"
    # Remove the legacy .list Chrome used to write, to avoid duplicate sources.
    if [[ -f /etc/apt/sources.list.d/google-chrome.list ]]; then
      sudo_run rm -f /etc/apt/sources.list.d/google-chrome.list
    fi
    want="### THIS FILE IS AUTOMATICALLY CONFIGURED ###
# Changes to this file will not be preserved.
# This file will not be recreated if removed.
X-Repolib-Name: Google Chrome
Types: deb
URIs: https://dl.google.com/linux/chrome-stable/deb/
Suites: stable
Components: main
Architectures: amd64
Signed-By: $CHROME_KEYRING"
    add_apt_source google-chrome "$want"
    apt_update_if_dirty
    apt_install google-chrome-stable
  fi

  # Theme color from the wallpaper (system/files/chrome/dot-chrome-theme):
  # a root-owned helper copies one validated color from the user's
  # colors.json into Chrome's managed policy whenever dot-theme rewrites it.
  # Installed on every arch, so it's in place if Chrome is added later.
  step "Chrome theme color follows the wallpaper"
  CHROME_POLICY_DIR=/etc/opt/chrome/policies/managed
  [[ -d $CHROME_POLICY_DIR ]] || sudo_run install -d -m 0755 "$CHROME_POLICY_DIR"
  install_root_file "$FILES_DIR/chrome/dot-chrome-theme" /usr/local/libexec/dot-chrome-theme 0755
  chrome_home="$(user_home "$TARGET_USER")"
  for unit in dot-chrome-theme.path dot-chrome-theme.service; do
    render_template "$FILES_DIR/chrome/$unit.in" "USER=$TARGET_USER" "HOME=$chrome_home" |
      write_root_file "/etc/systemd/system/$unit" 0644
  done
  sctl daemon-reload
  sctl enable --now dot-chrome-theme.path
  sctl start dot-chrome-theme.service
fi

step "Snapshot"
snap_create "drivers + apps"

step "Done"
info "Check: docker run --rm hello-world (after re-login); google-chrome --version"
info "Next: system/50-hyprland.sh"
