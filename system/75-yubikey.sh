#!/usr/bin/env bash
# Phase 7b: tools for the YubiKey C Bio (after 70-bitwarden.sh, which puts the
# key into sudo, polkit and the lock screen).
#   - fido2-tools (apt): fido2-token and fido2-assert, from the same libfido2
#     1.16 that Ubuntu's pam_u2f and pamu2fcfg use, so what they report is
#     what PAM sees. dot-u2f-enroll needs them.
#   - checks that udev gives you the key: systemd's 60-fido-id.rules tags FIDO
#     tokens ID_SECURITY_TOKEN, 70-uaccess.rules gives the seat user an ACL on
#     /dev/hidraw*. libu2f-udev is gone in 26.04; nothing else is needed.
#   - --piv: pcscd + pcsc-tools for the Multi-protocol Edition's PIV (smart
#     card), the optional group piv of system/packages.toml. Off by default:
#     the FIDO Edition has no smart card, and FIDO2 goes over hidraw without
#     pcscd. pcscd is socket-activated and exits when idle (--auto-exit), so
#     it only runs while something talks to a card. --piv and --no-piv are
#     kept, so kubaos update follows them.
#
# ykman (yubikey-manager) comes from home-manager (home/cli.nix), not apt:
# apt's yubikey-manager depends on pcscd, which the FIDO Edition never needs.
# Yubico Authenticator (GUI) isn't installed: ykman covers the PIN and the
# fingerprints, and the app wants pcscd for its main (OATH) screen. For a GUI
# anyway: nix run nixpkgs#yubioath-flutter. docs/SECRETS.md, "YubiKey C Bio".
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

PIV_CHOICE=""

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--piv | --no-piv]

  --piv      also install pcscd and pcsc-tools (Multi-protocol Edition: PIV / smart card),
             now and in every kubaos update
  --no-piv   leave them out from now on (kubaos packages disable piv removes them)
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --piv) PIV_CHOICE=on ;;
    --no-piv) PIV_CHOICE=off ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

require_sudo

# --------------------------------------------------------------------------
[[ -z "$PIV_CHOICE" ]] || choice_set piv "$PIV_CHOICE"
step "Packages"
group_install yubikey piv
if group_wanted piv; then
  # The socket, not the service: pcscd starts on the first client.
  sctl enable --now pcscd.socket
fi

# --------------------------------------------------------------------------
step "udev: access to the key without root"
rules_ok=1
for r in /usr/lib/udev/rules.d/60-fido-id.rules /usr/lib/udev/rules.d/70-uaccess.rules; do
  [[ -f $r ]] || { warn "$r missing (package udev / systemd)"; rules_ok=0; }
done
if ((rules_ok)) && grep -q 'ID_SECURITY_TOKEN.*uaccess' /usr/lib/udev/rules.d/70-uaccess.rules; then
  ok "systemd tags FIDO keys as security tokens and gives the seat user access"
else
  warn "no uaccess rule for ID_SECURITY_TOKEN: the lock screen and dot-u2f-enroll won't see the key"
fi

# --------------------------------------------------------------------------
step "Plugged-in YubiKeys"
found=0
for d in /sys/bus/usb/devices/*; do
  [[ -f $d/idVendor && $(cat "$d/idVendor") == 1050 ]] || continue
  found=1
  product="$(cat "$d/product" 2>/dev/null || echo YubiKey)"
  # The Multi-protocol Edition (and a YubiKey 5) adds a CCID interface
  # (class 0b): that's the smart card side.
  ccid=0
  for i in "$d"/*:*/bInterfaceClass; do
    [[ -f $i && $(cat "$i") == 0b ]] && ccid=1
  done
  if ((ccid)); then
    ok "$product ($(cat "$d/idProduct")): FIDO + smart card (PIV)"
    if ! group_wanted piv && ! pkg_installed pcscd; then
      info "For PIV (certificates, smart-card login): $0 --piv"
    fi
  else
    ok "$product ($(cat "$d/idProduct")): FIDO only"
  fi
  for h in /dev/hidraw*; do
    [[ -e $h ]] || continue
    udevadm info -q property -n "$h" 2>/dev/null | grep -q '^ID_SECURITY_TOKEN=1' || continue
    if ! command -v getfacl >/dev/null; then
      ok "$h is a FIDO security token (getfacl not installed, ACL not checked)"
    elif getfacl -p "$h" 2>/dev/null | grep -q "^user:$TARGET_USER:"; then
      ok "$h: $TARGET_USER has access (uaccess)"
    else
      info "$h: no ACL for $TARGET_USER yet (fine over ssh or in a container; it's set for the seat's active session)"
    fi
  done
done
((found)) || info "none plugged in (fine; plug it in and run dot-u2f-enroll --info)"

# --------------------------------------------------------------------------
step "Done"
cat <<EOF
      As yourself, with the key plugged in (docs/SECRETS.md, "YubiKey C Bio"):
        1. home-manager switch      ykman (yubikey-manager)
        2. ykman fido access change-pin
        3. ykman fido fingerprints add right-index   (2-3 fingers, one from the other hand)
        4. dot-u2f-enroll            then again with the backup key
EOF
