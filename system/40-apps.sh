#!/usr/bin/env bash
# Phase 4 (apps): Docker CE from Docker's repo, Google Chrome from Google's
# repo. Both are optional groups of system/packages.toml: --no-docker and
# --no-chrome turn them off for good, so kubaos update leaves them out too,
# and `kubaos packages enable docker|chrome` turns them on again.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

DO_DOCKER=1 DO_CHROME=1 NO_DOCKER=0 NO_CHROME=0
group_enabled docker || DO_DOCKER=0
group_enabled chrome || DO_CHROME=0

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--no-docker] [--no-chrome]

  --no-docker     leave Docker out, now and in every kubaos update
  --no-chrome     leave Chrome out, now and in every kubaos update
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --no-docker) DO_DOCKER=0 NO_DOCKER=1 ;;
    --no-chrome) DO_CHROME=0 NO_CHROME=1 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

require_sudo
if ((NO_DOCKER)); then choice_set docker off; fi
if ((NO_CHROME)); then choice_set chrome off; fi
group_install tools

# --------------------------------------------------------------------------
if ((DO_DOCKER)); then
  step "Docker CE from download.docker.com (suite resolute)"
  # Docker's install guide: remove distro packages that conflict.
  apt_purge docker.io docker-compose docker-compose-v2 docker-doc podman-docker containerd runc
  group_install docker

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
    # Remove the legacy .list Chrome used to write, to avoid duplicate sources.
    if [[ -f /etc/apt/sources.list.d/google-chrome.list ]]; then
      sudo_run rm -f /etc/apt/sources.list.d/google-chrome.list
    fi
    group_install chrome
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
