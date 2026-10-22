#!/usr/bin/env bash
# The update routine (docs/PLAN.md, "Update routine"):
#   1. snapper snapshot "before update"
#   2. apt update + upgrade (held packages stay put), report what's held back
#   3. --hyprland: unhold, upgrade Hyprland + hypr* libs, re-hold
#   4. --nix: nix flake update (all inputs or --input NAME) of the user flake
#      (~/.config/kubaos/home; a checkout's flake where there is none),
#      home-manager switch, relink the lock screen's pam_fprintd.so and
#      pam_u2f.so (lib.sh)
#   5. report newer Bitwarden desktop/CLI releases (pinned in 70-bitwarden.sh)
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

DO_APT=1 DO_HYPR=0 DO_NIX=0 ASSUME_YES=0
NIX_INPUTS=()
HM_USER="${USER:-$(id -un)}"
# The user flake the install wrote: your nixpkgs, KubaOS from
# /usr/share/kubaos. Without one (a machine set up by hand), this checkout.
USER_FLAKE="${XDG_CONFIG_HOME:-$HOME/.config}/kubaos/home"
if [[ -f "$USER_FLAKE/flake.nix" ]]; then FLAKE="$USER_FLAKE"; else FLAKE="$DOTFILES_DIR"; fi
case "$(hostname -s 2>/dev/null || true)" in
  vm) HM_HOST="vm" ;;
  *) HM_HOST="tuxedo" ;;
esac

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--yes] [--hyprland] [--nix [--input NAME]...] [--no-apt]
          [--user NAME] [--host NAME]

  --yes           don't ask before apt upgrade
  --hyprland      also upgrade the held Hyprland stack (read the release notes
                  first: https://github.com/hyprwm/Hyprland/releases)
  --nix           nix flake update + home-manager switch ($FLAKE)
  --input NAME    with --nix: update only this flake input (repeatable),
                  e.g. --input quickshell
  --no-apt        skip the apt part (e.g. only --nix)
  --user/--host   home-manager config <user>@<host> (default: $HM_USER@$HM_HOST)
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    -y | --yes) ASSUME_YES=1 ;;
    --hyprland) DO_HYPR=1 ;;
    --nix) DO_NIX=1 ;;
    --input) NIX_INPUTS+=("${2:?--input needs a name}"); shift ;;
    --no-apt) DO_APT=0 ;;
    --user) HM_USER="${2:?--user needs a name}"; shift ;;
    --host) HM_HOST="${2:?--host needs a name}"; shift ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

require_sudo

hypr_held() {
  apt-mark showhold 2>/dev/null | grep -E "$HYPR_HOLD_RE" || true
}

# --------------------------------------------------------------------------
step "Snapshot before update"
if snapper_ready; then
  sudo_run snapper -c root create --cleanup-algorithm number --description "before update $(date +%F)"
else
  warn "snapper not configured: updating without a snapshot, so no way back this time"
fi

# --------------------------------------------------------------------------
if ((DO_APT)); then
  step "apt update + upgrade"
  apt_update
  # --with-new-pkgs: plain apt-get upgrade keeps back anything that needs a new
  # package, which includes every kernel update (new linux-image-<version>).
  # Held packages still stay put.
  if ((ASSUME_YES)); then
    sudo_run apt-get upgrade --with-new-pkgs -y
  else
    sudo_run apt-get upgrade --with-new-pkgs
  fi

  step "Held packages"
  held="$(apt-mark showhold 2>/dev/null || true)"
  if [[ -z "$held" ]]; then
    info "none"
  else
    printf '%s\n' "$held" | sed 's/^/        /'
    mapfile -t held_pkgs <<<"$held"
    waiting="$(apt list --upgradable 2>/dev/null | grep -F -f <(printf '%s/\n' "${held_pkgs[@]}") || true)"
    if [[ -n "$waiting" ]]; then
      info "held back, newer version available:"
      printf '%s\n' "$waiting" | sed 's/^/        /'
      info "Hyprland: re-run with --hyprland after reading the release notes."
    else
      ok "held packages are current"
    fi
  fi
fi

# --------------------------------------------------------------------------
if ((DO_HYPR)); then
  step "Upgrade the held Hyprland stack"
  mapfile -t hypr < <(hypr_held)
  if ((${#hypr[@]} == 0)); then
    warn "no held hypr* packages found (did system/50-hyprland.sh run?)"
  else
    info "before: $(dpkg-query -W -f='${Version}' hyprland 2>/dev/null || echo '?')"
    # Re-hold whatever happens, so a failed upgrade never leaves them unheld.
    rehold() { sudo_run apt-mark hold "${hypr[@]}" >/dev/null; }
    trap rehold EXIT
    sudo_run apt-mark unhold "${hypr[@]}"
    ((DO_APT)) || apt_update
    # install --only-upgrade also pulls new library sonames (libaquamarineN+1).
    yes_flag=(); ((ASSUME_YES)) && yes_flag=(-y)
    sudo_run apt-get install --only-upgrade --no-install-recommends "${yes_flag[@]}" "${hypr[@]}"
    # Newly pulled hypr* libraries get held too.
    mapfile -t hypr < <({ printf '%s\n' "${hypr[@]}"; hypr_installed; } | sort -u)
    rehold
    trap - EXIT
    ok "re-held: ${hypr[*]}"
    info "after: $(dpkg-query -W -f='${Version}' hyprland 2>/dev/null || echo '?')"
    if command -v Hyprland >/dev/null && ((!DRY_RUN)); then
      if Hyprland --verify-config >/dev/null 2>&1; then
        ok "Hyprland --verify-config passes"
      else
        warn "Hyprland --verify-config reports problems: run it and fix config/hypr/ before logging out"
      fi
    fi
    info "Log out and back in to run the new Hyprland. If it breaks: Ctrl+Alt+F2,"
    info "roll back with the snapshot taken above (docs/INSTALL.md, Troubleshooting)."
  fi
fi

# --------------------------------------------------------------------------
if ((DO_NIX)); then
  step "Nix: flake update + home-manager switch"
  if ! command -v nix >/dev/null && [[ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]]; then
    set +u
    # shellcheck disable=SC1091
    . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
    set -u
  fi
  if ((DRY_RUN)) || command -v nix >/dev/null; then
    run nix flake update "${NIX_INPUTS[@]}" --flake "$FLAKE"
    if ((!DRY_RUN)) && [[ $FLAKE == "$DOTFILES_DIR" ]] && command -v git >/dev/null; then
      git -C "$DOTFILES_DIR" --no-pager diff --stat -- flake.lock || true
    fi
    # A release's new wallpaper sets (opt-in: dot-theme shows the chosen
    # ones). Before the switch, as in 60-nix.sh.
    link_wallpapers
    # -b hm-backup, as in 60-nix.sh: a file in the way is renamed, not fatal.
    if command -v home-manager >/dev/null; then
      run home-manager switch -b hm-backup --flake "$FLAKE#$HM_USER@$HM_HOST"
    else
      run nix run --inputs-from "$KUBAOS_DIR" home-manager -- \
        switch -b hm-backup --flake "$FLAKE#$HM_USER@$HM_HOST"
    fi
    # The lock screen's PAM modules follow KubaOS's nixpkgs (lib.sh), which
    # moves with a release, not with the update above; relinking is a no-op
    # until then.
    link_pam_fprintd
    if [[ -L "$PAM_U2F_LINK" ]] || ((DRY_RUN)); then link_pam_u2f; fi
    # gpu-screen-recorder's setcap'd KMS helper must match the new gsr.
    if [[ -e /usr/local/libexec/dotfiles/gsr-kms-server ]]; then
      rec_args=()
      ((DRY_RUN)) && rec_args+=(--dry-run)
      run "$SYSTEM_DIR/65-recorder.sh" "${rec_args[@]}"
    fi
    info "If activation warned about GPU drivers: system/60-nix.sh --no-switch --no-chsh --force-gpu"
    info "Broken? home-manager generations, then run <path>/activate of the previous one."
    [[ $FLAKE != "$DOTFILES_DIR" ]] || info "Happy? commit flake.lock."
  else
    warn "nix not found; skipping the Nix update (system/60-nix.sh installs it)"
  fi
fi

# --------------------------------------------------------------------------
# Read-only: the desktop app and bw are pinned .deb/binary downloads, not apt.
if ((DO_APT)) && command -v jq >/dev/null; then
  "$SYSTEM_DIR/70-bitwarden.sh" --check-updates || warn "Bitwarden release check failed"
fi

# --------------------------------------------------------------------------
step "Done"
if [[ -f /var/run/reboot-required ]]; then
  warn "reboot required: $(tr '\n' ' ' </var/run/reboot-required.pkgs 2>/dev/null || true)"
fi
