#!/usr/bin/env bash
# Phase 4 (hardware): TUXEDO apt repo + tuxedo-drivers (no Control Center),
# Secure Boot MOK for DKMS, power profile check, Thunderbolt/USB4 dock (bolt),
# fingerprint reader (fprintd + PAM).
#
# First-boot setup (kubaos setup, kubaos-setup) runs parts of this as
# root through its helper (installer/firstboot/setup-helper):
#   --only tuxedo,mok --mok-password-stdin   the TUXEDO card
#   --only fingerprint                       the fingerprint card
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

TUXEDO_KEY_URL="https://deb.tuxedocomputers.com/0x54840598.pub.asc"
TUXEDO_KEY_FPR="E5D0C320BBCE8D21CDF60DD5120ED28D54840598"
TUXEDO_KEYRING="/etc/apt/keyrings/tuxedo.gpg"
ENABLE_FPRINT_PAM=1
ONLY="tuxedo,mok,power,dock,fingerprint"
MOK_PW_STDIN=0 MOK_PW=""

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--no-fprint-pam] [--only LIST] [--mok-password-stdin]

  --no-fprint-pam        don't enable fingerprint auth in /etc/pam.d/common-auth
  --only LIST            just these parts, comma separated (default: all):
                         tuxedo (repo, headers, drivers), mok, power, dock, fingerprint
  --mok-password-stdin   read the one-time MOK password from the first line of
                         stdin instead of letting mokutil ask on the terminal
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --no-fprint-pam) ENABLE_FPRINT_PAM=0 ;;
    --only) ONLY="${2:?--only needs a list}"; shift ;;
    --mok-password-stdin) MOK_PW_STDIN=1 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done
want() { [[ ",$ONLY," == *",$1,"* ]]; }

if ((MOK_PW_STDIN)); then
  # Read it before apt runs: nothing else may see stdin.
  IFS= read -r MOK_PW || true
  [[ -n $MOK_PW ]] || die "--mok-password-stdin: no password on stdin"
  exec </dev/null
fi

# A fresh KubaOS install has no apt lists (the payload ships without them):
# fetch them, but only when something is actually missing, so the parts
# that are already installed work offline.
apt_install_fresh() {
  local p
  for p in "$@"; do
    if ! pkg_installed "$p"; then
      compgen -G '/var/lib/apt/lists/*_Packages' >/dev/null || apt_update
      break
    fi
  done
  apt_install "$@"
}

require_sudo
apt_install_fresh ca-certificates curl gnupg

# --------------------------------------------------------------------------
HAVE_TUXEDO=0
is_amd64 && HAVE_TUXEDO=1
if want tuxedo; then
step "TUXEDO apt repository (resolute)"
if ! is_amd64; then
  warn "architecture $(arch): TUXEDO publishes amd64/i386 only, skipping repo and drivers (VM?)"
else
  [[ -d /etc/apt/keyrings ]] || sudo_run install -d -m 0755 /etc/apt/keyrings
  fetch_key "$TUXEDO_KEY_URL" "$TUXEDO_KEYRING" "$TUXEDO_KEY_FPR"
  add_apt_source tuxedo "Types: deb
URIs: https://deb.tuxedocomputers.com/ubuntu
Suites: resolute
Components: main
Architectures: amd64
Signed-By: $TUXEDO_KEYRING"
  apt_update_if_dirty
fi

# --------------------------------------------------------------------------
step "Kernel headers + DKMS (tuxedo-drivers is a DKMS package)"
headers=(dkms mokutil)
kver="$(uname -r)"
if in_container; then
  info "container: running kernel $kver isn't Ubuntu's, skipping kernel headers"
else
  pkg_has_candidate "linux-headers-$kver" && headers+=("linux-headers-$kver")
  # keep headers coming for future kernels of the installed flavour
  pkg_installed linux-image-generic && headers+=(linux-headers-generic)
  pkg_installed linux-image-generic-hwe-26.04 && headers+=(linux-headers-generic-hwe-26.04)
fi
apt_install_fresh "${headers[@]}"

# --------------------------------------------------------------------------
if ((HAVE_TUXEDO)); then
  step "tuxedo-drivers (NOT tuxedo-control-center)"
  apt_install_fresh tuxedo-drivers udev-hid-bpf
  if pkg_installed tuxedo-control-center; then
    warn "tuxedo-control-center is installed; KubaOS uses power-profiles-daemon instead"
  fi
  if command -v dkms >/dev/null; then
    dkms status tuxedo-drivers 2>/dev/null | sed 's/^/        /' || true
  fi
  if ((!DRY_RUN)) && ! in_container; then
    loaded="$(lsmod | awk '$1 ~ /^(tuxedo|uniwill|clevo)/ {print $1}' | tr '\n' ' ')"
    if [[ -n "$loaded" ]]; then ok "modules loaded: $loaded"; else info "no tuxedo modules loaded yet (reboot, or MOK enrollment pending)"; fi
  fi
fi
fi

# --------------------------------------------------------------------------
# After the drivers: DKMS makes its signing key while it builds them, so one
# run installs and enrolls.
if want mok; then
step "Secure Boot: DKMS signing key (MOK)"
sb_state="$(mokutil --sb-state 2>/dev/null || true)"
if [[ -n "$sb_state" ]]; then info "$sb_state"; else info "mokutil can't read the Secure Boot state (no EFI vars?)"; fi
if [[ "$sb_state" == *"SecureBoot enabled"* ]]; then
  mok=""
  for c in /var/lib/shim-signed/mok/MOK.der /var/lib/dkms/mok.pub; do
    as_root test -f "$c" && { mok="$c"; break; }
  done
  # mokutil --test-key exits 1 when the key is already enrolled or pending,
  # so read its message instead of its exit code.
  mok_state=""
  [[ -n "$mok" ]] && mok_state="$(as_root mokutil --test-key "$mok" 2>&1 || true)"
  if [[ -z "$mok" ]]; then
    info "no DKMS signing key yet; dkms creates one when it builds the first module."
    info "re-run this script after installing tuxedo-drivers to enroll it."
    ((MOK_PW_STDIN)) && warn "no DKMS signing key to enroll yet: the tuxedo modules won't load with Secure Boot on"
  elif [[ "$mok_state" == *"is already enrolled"* || "$mok_state" == *"is already in db"* ]]; then
    ok "DKMS key $mok is enrolled"
  elif [[ "$mok_state" == *"already in the enrollment request"* ]]; then
    ok "DKMS key $mok is queued for enrollment: reboot and confirm it on the blue MokManager screen"
  elif [[ "$mok_state" != *"is not enrolled"* ]]; then
    warn "can't tell whether $mok is enrolled: $mok_state"
  else
    if ((MOK_PW_STDIN)); then
      info "DKMS key $mok is not enrolled yet. Queuing it with the password you picked."
      # mokutil asks twice; the password never reaches argv.
      printf '%s\n%s\n' "$MOK_PW" "$MOK_PW" | sudo_run mokutil --import "$mok"
    else
      warn "DKMS key $mok is not enrolled yet. You'll pick a one-time password now."
      sudo_run mokutil --import "$mok"
    fi
    info "On the next reboot the blue MokManager screen appears:"
    info "  Enroll MOK -> Continue -> Yes -> enter that password -> Reboot."
    info "Miss it and the tuxedo modules won't load. Run this script again."
  fi
else
  info "Secure Boot off: unsigned DKMS modules load as-is."
fi
fi

# --------------------------------------------------------------------------
if want power; then
step "Power profiles"
if [[ -r /sys/firmware/acpi/platform_profile_choices ]]; then
  choices="$(cat /sys/firmware/acpi/platform_profile_choices)"
  ok "platform_profile_choices: $choices"
  info "current: $(cat /sys/firmware/acpi/platform_profile 2>/dev/null || echo '?')"
  info "powerprofilesctl set performance  changes real firmware limits. Done."
else
  warn "/sys/firmware/acpi/platform_profile_choices is missing."
  info "After a reboot with tuxedo-drivers loaded it should exist. If it still"
  info "doesn't, power-profiles-daemon only tweaks the CPU governor: revisit"
  info "TUXEDO Control Center (docs/PLAN.md, Phase 4)."
fi
if command -v powerprofilesctl >/dev/null && has_systemd; then
  powerprofilesctl list 2>/dev/null | sed 's/^/        /' || true
fi
fi

# --------------------------------------------------------------------------
if want dock; then
step "USB4 / Thunderbolt dock (bolt)"
apt_install_fresh bolt
if command -v boltctl >/dev/null && has_systemd; then
  boltctl list 2>/dev/null | sed 's/^/        /' || true
fi
info "Plug the dock into the rear USB4 port, then:"
info "  boltctl list                      # find the dock's uuid"
info "  sudo boltctl enroll --policy auto <uuid>"
fi

# --------------------------------------------------------------------------
if want fingerprint; then
step "Fingerprint reader (fprintd)"
apt_install_fresh fprintd libpam-fprintd
if command -v lsusb >/dev/null; then
  lsusb 2>/dev/null | grep -iE 'finger|goodix|synaptics|elan|egis|focal|upek|validity' | sed 's/^/        reader? /' || info "no obvious fingerprint reader on USB (plug it in?)"
fi
info "Check the reader's USB id against https://fprint.freedesktop.org/supported-devices.html"
info "Enroll (interactive, as yourself, not root):"
info "  fprintd-enroll -f right-index-finger"
info "  fprintd-verify"

pam_mod=""
for d in /usr/lib/*-linux-gnu/security /lib/*-linux-gnu/security; do
  [[ -f "$d/pam_fprintd.so" ]] && { pam_mod="$d/pam_fprintd.so"; break; }
done
if ((!ENABLE_FPRINT_PAM)); then
  info "--no-fprint-pam: leaving PAM alone"
elif [[ -z "$pam_mod" || ! -f /usr/share/pam-configs/fprintd ]]; then
  warn "pam_fprintd module or its pam-auth-update profile is missing; not touching PAM"
elif grep -q pam_fprintd /etc/pam.d/common-auth 2>/dev/null; then
  ok "fingerprint already enabled in common-auth (sudo, polkit, tty login)"
else
  # Finger first, 10 s timeout, then the password prompt. Safe before
  # enrolling: with no enrolled finger the module steps aside immediately.
  sudo_run pam-auth-update --enable fprintd
fi
fi

step "Done"
info "Reboot now if tuxedo-drivers was just installed (and enroll the MOK if asked)."
info "Next: system/40-apps.sh"
