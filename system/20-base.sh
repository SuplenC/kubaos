#!/usr/bin/env bash
# Phase 3: base system. apt packages, NetworkManager instead of networkd, the
# Proxy's root helper, PipeWire, logind lid policy, short stop timeouts and nothing waiting for
# the network at boot, snapper on / with timers, grub-btrfs.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

GRUB_BTRFS_TAG="v4.14"   # https://github.com/Antynea/grub-btrfs/releases
GRUB_BTRFS_COMMIT="2fcfbe967637166b88dadd49c834807243a941bf"   # what $GRUB_BTRFS_TAG points at
APPLY_NETPLAN=1

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--no-netplan-apply]

Installs the base packages and plumbing. Safe to re-run.
  --no-netplan-apply   write the netplan file but don't apply it now
                       (automatic when connected over SSH; reboot instead)
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --no-netplan-apply) APPLY_NETPLAN=0 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

require_sudo

# The base system's groups of system/packages.toml.
BASE_GROUPS=(
  tools network proxy audio session graphics microcode laptop dock fingerprint
  build snapshots files fonts fallback shell
)

step "apt update"
apt_update

step "Install base packages (--no-install-recommends, system/packages.toml)"
group_install "${BASE_GROUPS[@]}"

# --------------------------------------------------------------------------
step "Network: netplan renders through NetworkManager"
NETPLAN_DEST=/etc/netplan/01-network-manager-all.yaml
netplan_changed=0
if ! same_content "$FILES_DIR/netplan/01-network-manager-all.yaml" "$NETPLAN_DEST"; then
  netplan_changed=1
fi
install_root_file "$FILES_DIR/netplan/01-network-manager-all.yaml" "$NETPLAN_DEST" 0600
# The installer (or cloud-init) wrote its own networkd config; park it. netplan
# reads only *.yaml, so the renamed file is kept for reference but ignored.
# 90-NM-*.yaml are NetworkManager's own saved connections (Ubuntu's NM stores
# its profiles through netplan): keep those, or a re-run forgets your Wi-Fi.
for f in /etc/netplan/*.yaml; do
  [[ -e "$f" && "$f" != "$NETPLAN_DEST" ]] || continue
  [[ "$(basename "$f")" == 90-NM-* ]] && continue
  info "disabling $f (renamed to $f.disabled-by-dotfiles)"
  sudo_run mv -- "$f" "$f.disabled-by-dotfiles"
  netplan_changed=1
done
if ((netplan_changed)); then
  if ! command -v netplan >/dev/null; then
    info "netplan not installed here, skipping generate/apply"
  elif ! has_systemd; then
    info "skip netplan apply (systemd not running here)"
  elif [[ -n "${SSH_CONNECTION:-}" || "$APPLY_NETPLAN" == 0 ]]; then
    sudo_run netplan generate
    warn "not applying netplan now (SSH session or --no-netplan-apply): reboot to switch to NetworkManager"
  else
    sudo_run netplan apply
  fi
else
  ok "netplan already hands everything to NetworkManager"
fi

if unit_exists NetworkManager.service; then
  if [[ "$(systemctl is-enabled NetworkManager.service 2>/dev/null || true)" == enabled ]]; then
    ok "NetworkManager enabled"
  else
    sctl enable --now NetworkManager.service
  fi
fi
for unit in systemd-networkd.service systemd-networkd.socket; do
  unit_exists "$unit" || continue
  state="$(systemctl is-enabled "$unit" 2>/dev/null || true)"
  if [[ "$state" =~ ^(disabled|masked)$ ]]; then ok "$unit is $state"; else sctl disable --now "$unit"; fi
done
if unit_exists systemd-networkd-wait-online.service; then
  if [[ "$(systemctl is-enabled systemd-networkd-wait-online.service 2>/dev/null || true)" == masked ]]; then
    ok "systemd-networkd-wait-online masked"
  else
    sctl disable --now systemd-networkd-wait-online.service || true
    sctl mask systemd-networkd-wait-online.service
  fi
fi
info "Wi-Fi: nmtui (or: nmcli device wifi connect <SSID> --ask), then unplug the cable."

# --------------------------------------------------------------------------
step "Proxy: the work network's root helper (polkit, pkexec)"
install_proxy_helper

# --------------------------------------------------------------------------
step "Audio: PipeWire user services (global enable for all users)"
if command -v systemctl >/dev/null; then
  for unit in pipewire.socket pipewire-pulse.socket wireplumber.service; do
    if [[ "$(systemctl --global is-enabled "$unit" 2>/dev/null || true)" == enabled ]]; then
      ok "$unit enabled (user)"
    else
      # --global only creates symlinks under /etc/systemd/user, works offline.
      sudo_run systemctl --global enable "$unit"
    fi
  done
else
  info "systemctl not available, skipping"
fi
if unit_exists bluetooth.service; then
  if [[ "$(systemctl is-enabled bluetooth.service 2>/dev/null || true)" == enabled ]]; then
    ok "bluetooth enabled"
  else
    sctl enable --now bluetooth.service
  fi
fi

# --------------------------------------------------------------------------
step "ddcutil: let $TARGET_USER talk to external monitors (i2c group)"
if getent group i2c >/dev/null; then
  if id -nG "$TARGET_USER" 2>/dev/null | tr ' ' '\n' | grep -qx i2c; then
    ok "$TARGET_USER is in i2c"
  elif id "$TARGET_USER" >/dev/null 2>&1; then
    sudo_run usermod -aG i2c "$TARGET_USER"
  fi
else
  info "no i2c group (ddcutil udev rules create it on install); skipping"
fi

# --------------------------------------------------------------------------
step "Lid + power key policy: logind drop-in (suspend on battery only)"
install_root_file "$FILES_DIR/logind.conf.d/lid.conf" /etc/systemd/logind.conf.d/lid.conf 0644
info "takes effect after the next reboot (restarting logind would end your session)"

# --------------------------------------------------------------------------
step "inotify: room for every program that watches files"
install_inotify_limits

# --------------------------------------------------------------------------
step "Short shutdown and boot: 10 s stop timeouts, no wait for the network, no console palette"
install_fast_shutdown_boot

# --------------------------------------------------------------------------
step "Snapper: config \"root\" for /"
root_fs="$(findmnt -no FSTYPE / 2>/dev/null || true)"
SKIP_BTRFS=0
if ((IMAGE_BUILD)); then
  # The payload's / ends up on btrfs; the install creates the config there.
  info "image build: grub-btrfs and the timers only; the install creates the snapper config"
elif [[ "$root_fs" != btrfs ]]; then
  warn "/ is $root_fs, not btrfs: skipping snapper and grub-btrfs"
  SKIP_BTRFS=1
fi

if ((!SKIP_BTRFS)); then
  if ((IMAGE_BUILD)); then
    :
  elif [[ -f /etc/snapper/configs/root ]]; then
    ok "snapper config root exists"
  elif findmnt --fstab -n /.snapshots >/dev/null 2>&1; then
    # /.snapshots is the @snapshots subvolume (00-btrfs-subvolumes.md).
    # snapper create-config insists on creating /.snapshots itself, so: let it,
    # then swap its nested subvolume for the @snapshots mount again. Checked
    # against fstab, not the live mount, so a re-run after a failure half-way
    # lands here again instead of falling back to a nested .snapshots.
    info "/.snapshots is the @snapshots mount in fstab; creating config around it"
    if findmnt -n /.snapshots >/dev/null 2>&1; then sudo_run umount /.snapshots; fi
    if [[ -d /.snapshots ]]; then sudo_run rmdir /.snapshots; fi
    sudo_run snapper -c root create-config /
    sudo_run btrfs subvolume delete /.snapshots
    sudo_run mkdir /.snapshots
    sudo_run mount /.snapshots
    sudo_run chmod 750 /.snapshots
  else
    warn "/.snapshots is not a separate @snapshots mount (see system/00-btrfs-subvolumes.md)."
    warn "Creating snapper's default nested .snapshots instead; rollback will be harder."
    sudo_run snapper -c root create-config /
  fi

  step "Snapper: retention and permissions"
  # Hourly timeline, keep a few days back; apt runs add pre/post pairs via
  # /etc/apt/apt.conf.d/80snapper (number cleanup).
  want=(
    "ALLOW_USERS=$TARGET_USER" "SYNC_ACL=yes"
    "TIMELINE_CREATE=yes" "TIMELINE_CLEANUP=yes"
    "TIMELINE_LIMIT_HOURLY=6" "TIMELINE_LIMIT_DAILY=7" "TIMELINE_LIMIT_WEEKLY=2"
    "TIMELINE_LIMIT_MONTHLY=1" "TIMELINE_LIMIT_YEARLY=0"
    "NUMBER_CLEANUP=yes" "NUMBER_LIMIT=30" "NUMBER_LIMIT_IMPORTANT=10"
  )
  todo=()
  for kv in "${want[@]}"; do
    if [[ -f /etc/snapper/configs/root ]] && as_root grep -qx "${kv%%=*}=\"${kv#*=}\"" /etc/snapper/configs/root; then
      continue
    fi
    todo+=("$kv")
  done
  if ((IMAGE_BUILD)); then
    # The install runs `snapper -c root set-config $(cat this file)` after create-config.
    printf '%s\n' "${want[@]}" | write_root_file /usr/share/kubaos-image/snapper-root.conf 0644
  elif ((${#todo[@]})); then
    sudo_run snapper -c root set-config "${todo[@]}"
  else
    ok "snapper config already set"
  fi

  for unit in snapper-timeline.timer snapper-cleanup.timer; do
    if [[ "$(systemctl is-enabled "$unit" 2>/dev/null || true)" == enabled ]]; then
      ok "$unit enabled"
    else
      sctl enable --now "$unit"
    fi
  done

  # ------------------------------------------------------------------------
  step "grub-btrfs: snapshots in the GRUB menu"
  if pkg_installed grub-btrfs; then
    ok "grub-btrfs installed from apt"
  elif pkg_has_candidate grub-btrfs; then
    apt_install grub-btrfs
  elif [[ -x /etc/grub.d/41_snapshots-btrfs ]] && command -v grub-btrfsd >/dev/null; then
    ok "grub-btrfs installed from source ($(grep -m1 '^GRUB_BTRFS_VERSION' /etc/default/grub-btrfs/config 2>/dev/null || echo 'version unknown'))"
  elif ! command -v update-grub >/dev/null; then
    warn "GRUB not installed here, skipping grub-btrfs"
  else
    # Not in the Ubuntu archive (checked 2026-10 for resolute). Build the
    # pinned upstream release; it installs to /usr and /etc/grub.d.
    info "grub-btrfs is not packaged for resolute, installing upstream $GRUB_BTRFS_TAG"
    apt_install make
    src="$(mktemp -d)"
    run git -c advice.detachedHead=false clone --quiet --depth 1 --branch "$GRUB_BTRFS_TAG" \
      https://github.com/Antynea/grub-btrfs.git "$src/grub-btrfs"
    # It runs as root: build only the commit we reviewed, even if the tag moves.
    if ((!DRY_RUN)); then
      head="$(git -C "$src/grub-btrfs" rev-parse HEAD)"
      if [[ "$head" != "$GRUB_BTRFS_COMMIT" ]]; then
        rm -rf "$src"
        die "grub-btrfs $GRUB_BTRFS_TAG moved to $head (reviewed: $GRUB_BTRFS_COMMIT). Not running it as root; review it and update the pin"
      fi
    fi
    # GRUB_UPDATE_EXCLUDE: update-grub runs once below anyway.
    sudo_run make -C "$src/grub-btrfs" install INSTALL_DOCS=false GRUB_UPDATE_EXCLUDE=true
    rm -rf "$src"
    info "to uninstall later: sudo make -C <checkout> uninstall"
  fi

  # Submenu title. grub-btrfs would derive it from the distro name; spell it
  # out so the menu says KubaOS whatever os-release ends up saying.
  btrfs_cfg=/etc/default/grub-btrfs/config
  submenu='GRUB_BTRFS_SUBMENUNAME="KubaOS snapshots"'
  if [[ ! -f "$btrfs_cfg" ]]; then
    info "no $btrfs_cfg yet (dry-run or no GRUB); submenu name not set"
  elif grep -qxF "$submenu" "$btrfs_cfg"; then
    ok "GRUB submenu is \"KubaOS snapshots\""
  else
    sudo_run sed -i -e '/^GRUB_BTRFS_SUBMENUNAME=/d' -e "\$a $submenu" "$btrfs_cfg"
  fi

  if unit_exists grub-btrfsd.service; then
    if [[ "$(systemctl is-enabled grub-btrfsd.service 2>/dev/null || true)" == enabled ]]; then
      ok "grub-btrfsd enabled (regenerates the menu when snapshots change)"
    else
      sctl enable --now grub-btrfsd.service
    fi
  else
    info "grub-btrfsd.service not present yet (dry-run or no GRUB)"
  fi
  if command -v update-grub >/dev/null && has_systemd; then
    sudo_run update-grub
  fi

  step "First snapshot"
  snap_create "clean base"
fi

step "Done"
info "Check: nmcli general status; wpctl status; snapper -c root list; ls /etc/grub.d/41_snapshots-btrfs"
info "Next: system/30-hardware.sh"
