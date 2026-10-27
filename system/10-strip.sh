#!/usr/bin/env bash
# Phase 2: strip the server parts. Purges snapd, cloud-init, multipath-tools,
# lxd-installer and ModemManager (and with them the ubuntu-server*
# metapackages, after marking their other dependencies manual), pins snapd out
# for good, silences the Ubuntu Pro adverts (apt news, ESM hints, motd news),
# keeps unattended-upgrades.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

usage() {
  cat <<EOF
Usage: $0 [--dry-run]

Removes the server-only packages and the Ubuntu Pro adverts. Safe to re-run.
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

PURGE=(snapd cloud-init multipath-tools lxd-installer modemmanager)

step "Keep what the server metapackages pulled in"
# ubuntu-server-minimal depends on snapd, cloud-init and multipath-tools, and
# ubuntu-server on lxd-installer and multipath-tools, so the purge below
# removes both metapackages. Everything else they pulled in (sudo, cryptsetup,
# lvm2, systemd-resolved, ...) is marked auto-installed, and --auto-remove
# would take it along. Mark those manual first.
metas=()
for m in ubuntu-server ubuntu-server-minimal; do
  if pkg_installed "$m"; then metas+=("$m"); fi
done
if ((${#metas[@]})); then
  mapfile -t auto_pkgs < <(apt-mark showauto 2>/dev/null)
  mapfile -t keep < <(
    apt-cache depends --no-suggests --no-conflicts --no-breaks --no-replaces --no-enhances "${metas[@]}" 2>/dev/null |
      awk '$1 ~ /^\|?(PreDepends|Depends|Recommends):$/ && $2 !~ /^</ {print $2}' | sort -u |
      grep -vxF -f <(printf '%s\n' "${PURGE[@]}") |
      grep -xF -f <(printf '%s\n' "${auto_pkgs[@]}")
  )
  if ((${#keep[@]})); then
    sudo_run apt-mark manual "${keep[@]}"
  else
    ok "dependencies of ${metas[*]} are already marked manual"
  fi
else
  ok "no server metapackages installed"
fi

step "Purge server packages: ${PURGE[*]}"
if pkg_installed snapd && command -v snap >/dev/null; then
  info "installed snaps (purging snapd removes them):"
  snap list 2>/dev/null | sed 's/^/        /' || true
fi
apt_purge "${PURGE[@]}"

step "Remove leftovers of snapd and cloud-init"
leftovers=(/etc/cloud /var/lib/cloud /snap /var/snap /var/lib/snapd /var/cache/snapd /root/snap)
home="$(user_home "$TARGET_USER")"
[[ -n "$home" ]] && leftovers+=("$home/snap")
found=0
for d in "${leftovers[@]}"; do
  if as_root test -e "$d"; then
    found=1
    sudo_run rm -rf -- "$d"
  fi
done
((found)) || ok "nothing left over"

step "Keep snapd from coming back (apt pin + hold)"
install_root_file "$FILES_DIR/apt/preferences.d/no-snapd" /etc/apt/preferences.d/no-snapd 0644
if apt-mark showhold 2>/dev/null | grep -qx snapd; then
  ok "snapd is on hold"
elif ! dpkg-query -W snapd >/dev/null 2>&1; then
  # Once purged and pinned to -10, apt has no candidate and dpkg no record,
  # so there's nothing to hold; the pin is what blocks it.
  ok "snapd unknown to dpkg and pinned out (nothing to hold)"
else
  sudo_run apt-mark hold snapd || warn "apt-mark hold snapd failed, the pin still blocks it"
fi

step "Ubuntu Pro ads off: apt news and ESM hints"
if command -v pro >/dev/null; then
  if [[ "$(pro config show apt_news 2>/dev/null | awk '{print $2}')" == False ]]; then
    ok "pro apt_news is already false"
  else
    sudo_run pro config set apt_news=false || warn "pro config set apt_news=false failed"
  fi
else
  ok "ubuntu-pro-client not installed"
fi
for unit in apt-news.service esm-cache.service; do
  if unit_exists "$unit"; then
    if [[ "$(systemctl is-enabled "$unit" 2>/dev/null || true)" == masked ]]; then
      ok "$unit is masked"
    else
      sctl mask "$unit"
    fi
  fi
done
# The ESM status line in the login motd.
if [[ -x /etc/update-motd.d/91-contract-ua-esm-status ]]; then
  sudo_run chmod -x /etc/update-motd.d/91-contract-ua-esm-status
  info "a ubuntu-pro-client upgrade may put the exec bit back; re-run this script then"
fi

step "Disable motd-news (no ads at login either)"
write_root_file /etc/default/motd-news 0644 <<'EOF'
# Written by dotfiles/system/10-strip.sh: no news fetched from motd.ubuntu.com.
ENABLED=0
EOF
if unit_exists motd-news.timer; then
  if [[ "$(systemctl is-enabled motd-news.timer 2>/dev/null || true)" =~ ^(disabled|masked)$ ]]; then
    ok "motd-news.timer is off"
  else
    sctl disable --now motd-news.timer
  fi
fi

step "Retire Ubuntu's updaters (only kubaos update runs apt)"
retire_ubuntu_updaters

step "Done"
info "Next: system/20-base.sh"
