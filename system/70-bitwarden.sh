#!/usr/bin/env bash
# Phase 7: Bitwarden and the security key.
#   - Bitwarden desktop (official .deb, pinned + sha256-checked): its SSH agent
#     serves the SSH keys from the vault on ~/.bitwarden-ssh-agent.sock
#   - bw, the official CLI (standalone binary, pinned + sha256-checked) to
#     /usr/local/bin/bw, for `kubaos bitwarden`
#   - libsecret-tools (secret-tool) for `kubaos bitwarden env`'s keyring cache
#   - pam_u2f: a FIDO2 security key (a YubiKey C Bio: your finger on its
#     sensor) as an alternative to the password for sudo, polkit and the
#     Quickshell lock screen. The password always keeps working.
#     system/75-yubikey.sh adds the key's tools (fido2-tools, optional pcscd).
#
# Then, as yourself: kubaos bitwarden setup (vault -> ssh, git, gh, env) and
# dot-u2f-enroll (security keys). docs/SECRETS.md has the whole flow.
#
# Why these builds (checked 2026-10):
#   desktop  The .deb ships an AppArmor profile for Ubuntu 24.04+ (Electron's
#            sandbox needs unprivileged user namespaces), installs to
#            /opt/Bitwarden and is the build Bitwarden documents the SSH agent
#            and "unlock with system authentication" (polkit) for. There's no
#            apt repo and the app doesn't self-update on Linux, so the version
#            is pinned here like everything else. nixpkgs 26.05 has 2026.8.0,
#            but a Nix-built Electron on Ubuntu has no AppArmor profile for its
#            /nix/store path and nobody tests it there. amd64 only: Bitwarden
#            publishes no arm64 .deb (the arm64 VM skips the desktop app).
#   bw       nixpkgs 26.05 is stuck at 2026.4.2, six months behind the server.
#            The official standalone binary exists for amd64 and arm64,
#            starts as fast, and 2026.10 can unlock through the desktop app.
#
# Updating: kubaos update lists newer releases (this script's
# --check-updates); bump the versions and sha256s below, re-run this script.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

BW_DESKTOP_VERSION="2026.10.0"
BW_DESKTOP_SHA256="fcabfc7da3c327e2625b105e951cee4e60703bb8e256f0aac9cc9bc83ec7063f"   # Bitwarden-2026.10.0-amd64.deb
BW_CLI_VERSION="2026.10.0"
BW_CLI_SHA256_AMD64="269b67559caab860d1737f7554b32bdb8be9285d5689fee55193c137a0df9973"   # bw-linux-2026.10.0.zip
BW_CLI_SHA256_ARM64="2fe9d8a18dc4c35ea2f9e3e42a4021f14f129ef47e80b6cea1e3b34176f1583a"   # bw-linux-arm64-2026.10.0.zip
RELEASES="https://github.com/bitwarden/clients/releases/download"

U2F_MAPPINGS=/etc/u2f_mappings
# Services that accept the key. Each gets one line above `@include common-auth`:
# `sufficient` means a good key ends the stack, anything else (no key, wrong
# key, timeout) falls through to the unchanged password lines below it.
U2F_SERVICES=(sudo sudo-i polkit-1)
U2F_MARK="# dotfiles (system/70-bitwarden.sh): security key, the password below stays the fallback"
# Two lines, pam_u2f(8)'s "built-in UV with a fallback to PIN":
#   1. user verification: the finger on the Bio's sensor. Keys with no
#      fingerprint enrolled (or no reader) are skipped without a prompt.
#   2. the key's PIN, then a touch. Reached after the finger failed: the Bio
#      wants its PIN after 3 bad scans, and a good PIN clears that. Also how a
#      PIN-only backup key (dot-u2f-enroll --pin) signs in. Enter at the PIN
#      prompt skips it (libfido2 rejects a PIN under 4 characters before it
#      reaches the key, so no PIN try is used up).
# No key plugged in: both fail in milliseconds, nothing is printed. pam_u2f
# 1.4.0 (resolute). Brackets keep the spaces in cue_prompt.
U2F_LINES=(
  "auth       sufficient   pam_u2f.so authfile=$U2F_MAPPINGS cue nouserok userverification=1 pinverification=0 [cue_prompt=Touch the fingerprint sensor]"
  "auth       sufficient   pam_u2f.so authfile=$U2F_MAPPINGS cue nouserok userverification=0 pinverification=1 [cue_prompt=Touch the key to confirm the PIN]"
)
POLKIT_DROPIN=/etc/systemd/system/polkit-agent-helper@.service.d/u2f.conf

DO_DESKTOP=1 DO_CLI=1 DO_U2F=1 REMOVE_U2F=0 CHECK_ONLY=0

usage() {
  cat <<EOF
Usage: $0 [--dry-run] [--no-desktop] [--no-cli] [--no-u2f] [--remove-u2f] [--check-updates]

  --no-desktop      skip the Bitwarden desktop app
  --no-cli          skip the bw CLI
  --no-u2f          skip pam_u2f (packages, PAM lines, lock screen service)
  --remove-u2f      take the security-key lines out of /etc/pam.d/{sudo,sudo-i,polkit-1}
                    and remove the polkit drop-in (keeps $U2F_MAPPINGS)
  --check-updates   only report newer desktop/CLI releases on GitHub, change nothing
$(common_usage)
EOF
}

while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --no-desktop) DO_DESKTOP=0 ;;
    --no-cli) DO_CLI=0 ;;
    --no-u2f) DO_U2F=0 ;;
    --remove-u2f) REMOVE_U2F=1 ;;
    --check-updates) CHECK_ONLY=1 ;;
    -h | --help) usage; exit 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
  shift
done

# latest_tag PREFIX: newest release tag starting with PREFIX (cli-v, desktop-v).
# The repo releases web, browser, desktop and CLI from one tag list, so page
# through the newest 100 rather than asking for /releases/latest.
latest_tag() {
  curl -fsSL "https://api.github.com/repos/bitwarden/clients/releases?per_page=100" |
    jq -r --arg p "$1" '[.[] | select(.draft | not) | select(.prerelease | not) | .tag_name | select(startswith($p))] | first // empty'
}

# asset_sha256 TAG NAME: the sha256 GitHub records for a release asset.
asset_sha256() {
  curl -fsSL "https://api.github.com/repos/bitwarden/clients/releases/tags/$1" |
    jq -r --arg n "$2" '.assets[] | select(.name == $n) | .digest // empty | sub("^sha256:"; "")'
}

if ((CHECK_ONLY)); then
  step "Bitwarden releases on GitHub"
  command -v jq >/dev/null || die "jq not found: it comes from home-manager, run system/60-nix.sh first"
  cli="$(latest_tag cli-v)" || die "can't reach the GitHub API (offline? rate-limited?)"
  desk="$(latest_tag desktop-v)" || die "can't reach the GitHub API (offline? rate-limited?)"
  cli="${cli#cli-v}" desk="${desk#desktop-v}"
  if [[ -z "$cli" || -z "$desk" ]]; then
    warn "couldn't read the release list (rate-limited?)"
    exit 0
  fi
  if [[ "$cli" == "$BW_CLI_VERSION" ]]; then
    ok "bw CLI $BW_CLI_VERSION is the latest"
  else
    warn "bw CLI $cli is out (pinned: $BW_CLI_VERSION). New pins for system/70-bitwarden.sh:"
    info "BW_CLI_VERSION=\"$cli\""
    info "BW_CLI_SHA256_AMD64=\"$(asset_sha256 "cli-v$cli" "bw-linux-$cli.zip")\""
    info "BW_CLI_SHA256_ARM64=\"$(asset_sha256 "cli-v$cli" "bw-linux-arm64-$cli.zip")\""
  fi
  if [[ "$desk" == "$BW_DESKTOP_VERSION" ]]; then
    ok "Bitwarden desktop $BW_DESKTOP_VERSION is the latest"
  else
    warn "Bitwarden desktop $desk is out (pinned: $BW_DESKTOP_VERSION). New pins:"
    info "BW_DESKTOP_VERSION=\"$desk\""
    info "BW_DESKTOP_SHA256=\"$(asset_sha256 "desktop-v$desk" "Bitwarden-$desk-amd64.deb")\""
  fi
  info "Release notes: https://github.com/bitwarden/clients/releases"
  exit 0
fi

require_sudo

# fetch_verified URL SHA256 DEST: download to DEST, refuse anything that
# doesn't match the pinned hash.
fetch_verified() {
  local url="$1" sha="$2" dest="$3"
  run curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$dest"
  ((DRY_RUN)) && return 0
  if ! sha256sum --quiet -c <<<"$sha  $dest"; then
    rm -f "$dest"
    die "$url doesn't match the pinned sha256, not installing it"
  fi
}

# --------------------------------------------------------------------------
pkgs=(ca-certificates curl unzip libsecret-tools)
((DO_U2F && !REMOVE_U2F)) && pkgs+=(libpam-u2f pamu2fcfg pamtester)   # pamtester: docs/SECRETS.md, Troubleshooting
step "Packages"
apt_install "${pkgs[@]}"

# --------------------------------------------------------------------------
if ((DO_DESKTOP && !REMOVE_U2F)); then
  step "Bitwarden desktop $BW_DESKTOP_VERSION (.deb from github.com/bitwarden/clients)"
  have="$(dpkg-query -W -f='${Version}' bitwarden 2>/dev/null || true)"
  if ! is_amd64; then
    warn "$(arch): Bitwarden publishes no .deb for it; skipping the desktop app (no SSH agent here)"
  elif [[ "$have" == "$BW_DESKTOP_VERSION" ]]; then
    ok "bitwarden $have installed"
  else
    [[ -n "$have" ]] && info "installed: $have"
    tmp="$(mktemp -d)"
    fetch_verified "$RELEASES/desktop-v$BW_DESKTOP_VERSION/Bitwarden-$BW_DESKTOP_VERSION-amd64.deb" \
      "$BW_DESKTOP_SHA256" "$tmp/bitwarden.deb"
    # apt resolves the .deb's dependencies (libnss3, libnotify4, ...). _apt
    # can't read a mktemp dir; that only costs a harmless warning.
    sudo_run apt-get install -y --no-install-recommends "$tmp/bitwarden.deb"
    rm -rf "$tmp"
  fi
fi

# --------------------------------------------------------------------------
if ((DO_CLI && !REMOVE_U2F)); then
  step "bw CLI $BW_CLI_VERSION (/usr/local/bin/bw)"
  case "$(arch)" in
    amd64) cli_zip="bw-linux-$BW_CLI_VERSION.zip" cli_sha="$BW_CLI_SHA256_AMD64" ;;
    arm64) cli_zip="bw-linux-arm64-$BW_CLI_VERSION.zip" cli_sha="$BW_CLI_SHA256_ARM64" ;;
    *) die "no bw CLI build for $(arch)" ;;
  esac
  # Read the version from bw's own --version (about a second).
  have="$(/usr/local/bin/bw --version 2>/dev/null || true)"
  if [[ "$have" == "$BW_CLI_VERSION" ]]; then
    ok "bw $have"
  else
    [[ -n "$have" ]] && info "installed: $have"
    tmp="$(mktemp -d)"
    fetch_verified "$RELEASES/cli-v$BW_CLI_VERSION/$cli_zip" "$cli_sha" "$tmp/bw.zip"
    run unzip -q -o "$tmp/bw.zip" bw -d "$tmp"
    # Root-owned, outside $HOME: what reads your master password isn't
    # something any process running as you can swap.
    sudo_run install -D -m 0755 "$tmp/bw" /usr/local/bin/bw
    rm -rf "$tmp"
  fi
fi

# --------------------------------------------------------------------------
# pam_file_source SERVICE: the file PAM reads for SERVICE. polkitd ships its
# file in /usr/lib/pam.d; a file of the same name in /etc/pam.d replaces it.
pam_file_source() {
  if [[ -f "/etc/pam.d/$1" ]]; then echo "/etc/pam.d/$1"; else echo "/usr/lib/pam.d/$1"; fi
}

# u2f_render SRC: SRC with the security-key lines above the first auth line
# (the `@include common-auth` in all three files).
u2f_render() {
  local lines
  lines="$(printf '%s\n' "${U2F_LINES[@]}")"
  awk -v mark="$U2F_MARK" -v lines="$lines" '
    !done && ($1 == "@include" && $2 == "common-auth" || $1 == "auth") { print mark; print lines; done = 1 }
    { print }
    END { if (!done) exit 3 }' "$1"
}

# u2f_strip SRC: SRC without our mark and the pam_u2f lines under it (one
# line in older versions of this script, two now).
u2f_strip() {
  awk -v mark="$U2F_MARK" '$0 == mark { skip = 1; next } skip && /pam_u2f\.so/ { next } { skip = 0; print }' "$1"
}

if ((REMOVE_U2F)); then
  step "Remove the security key from ${U2F_SERVICES[*]}"
  for svc in "${U2F_SERVICES[@]}"; do
    f="/etc/pam.d/$svc"
    if [[ ! -f "$f" ]] || ! grep -qxF "$U2F_MARK" "$f"; then
      ok "$f has no security-key line"
    elif [[ -f "/usr/lib/pam.d/$svc" ]] && cmp -s <(u2f_strip "$f") "/usr/lib/pam.d/$svc"; then
      # Only a copy of the vendor file plus our lines: drop it, so package
      # updates to /usr/lib/pam.d/$svc apply again.
      sudo_run rm -f "$f"
    else
      u2f_strip "$f" | write_root_file "$f" 0644
    fi
  done
  if [[ -f "$POLKIT_DROPIN" ]]; then
    sudo_run rm -f "$POLKIT_DROPIN"
    has_systemd && sudo_run systemctl daemon-reload
  fi
  info "Left alone: $U2F_MAPPINGS, /etc/pam.d/qs-lock-u2f (turn off Settings.u2f for the lock screen)."
elif ((DO_U2F)); then
  step "Security key mappings ($U2F_MAPPINGS, root:root 0644)"
  # Central and root-owned on purpose: with ~/.config/Yubico/u2f_keys any
  # process running as you could add its own credential and get sudo without
  # your password. World-readable because the lock screen runs PAM as you.
  if [[ -f "$U2F_MAPPINGS" ]]; then
    if [[ "$(stat -c '%U:%G %a' "$U2F_MAPPINGS")" == "root:root 644" ]]; then
      ok "$U2F_MAPPINGS present ($(grep -c . "$U2F_MAPPINGS" || true) user line(s))"
    else
      sudo_run chown root:root "$U2F_MAPPINGS"
      sudo_run chmod 0644 "$U2F_MAPPINGS"
    fi
  else
    : | write_root_file "$U2F_MAPPINGS" 0644
  fi
  info "Enroll keys with dot-u2f-enroll (two keys: one is the backup)."

  step "pam_u2f for ${U2F_SERVICES[*]} (password stays as fallback)"
  for svc in "${U2F_SERVICES[@]}"; do
    src="$(pam_file_source "$svc")"
    if [[ ! -f "$src" ]]; then
      warn "no PAM file for $svc ($src), skipping"
      continue
    fi
    # Re-render from the file without our lines, so an older or edited line
    # is replaced and a current one is left alone.
    tmp="$(mktemp)"
    u2f_strip "$src" >"$tmp"
    if ! u2f_render "$tmp" >"$tmp.new"; then
      rm -f "$tmp" "$tmp.new"
      warn "/etc/pam.d/$svc has no auth line to put the key above; left alone"
      continue
    fi
    if [[ "$src" == "/etc/pam.d/$svc" ]] && cmp -s "$tmp.new" "$src"; then
      ok "/etc/pam.d/$svc has the security-key lines"
      rm -f "$tmp" "$tmp.new"
      continue
    fi
    write_root_file "/etc/pam.d/$svc" 0644 <"$tmp.new"
    rm -f "$tmp" "$tmp.new"
  done
  # A missing module would show up as "Module is unknown" on every sudo,
  # not as a wrong password, so check it's where Ubuntu's libpam looks.
  if ((!DRY_RUN)) && ! compgen -G "/usr/lib/*-linux-gnu/security/pam_u2f.so" >/dev/null; then
    warn "pam_u2f.so not found under /usr/lib/*-linux-gnu/security: is libpam-u2f installed?"
  fi
  info "Test from a second tty: sudo -k; sudo true  (no key: password prompt as before)"
  info "With the key: finger on the sensor; else its PIN (Enter skips), then the password."

  step "polkit helper can reach the key ($POLKIT_DROPIN)"
  before="$(cat "$POLKIT_DROPIN" 2>/dev/null || true)"
  install_root_file "$FILES_DIR/systemd/polkit-agent-helper@.service.d/u2f.conf" "$POLKIT_DROPIN" 0644
  if [[ "$before" != "$(cat "$POLKIT_DROPIN" 2>/dev/null || true)" ]] && has_systemd; then
    sudo_run systemctl daemon-reload
  fi

  step "Lock screen: /etc/pam.d/qs-lock-u2f and nixpkgs' pam_u2f.so ($PAM_U2F_LINK)"
  install_root_file "$FILES_DIR/pam.d/qs-lock-u2f" /etc/pam.d/qs-lock-u2f 0644
  if ((DRY_RUN)) || command -v nix >/dev/null; then
    link_pam_u2f
  else
    warn "nix not found (system/60-nix.sh first); the lock screen's key half stays off until then"
  fi
fi

# --------------------------------------------------------------------------
step "Done"
if ((!REMOVE_U2F)); then
  cat <<EOF
      As yourself (docs/SECRETS.md):
        1. Start Bitwarden, log in, Settings -> enable "SSH agent".
           Optional: "Unlock with system authentication" (polkit: your finger on the key).
        2. kubaos bitwarden setup  vault -> ~/.ssh, git signing, gh, project env
        3. system/75-yubikey.sh    the key's tools; then set its PIN and fingerprints
        4. dot-u2f-enroll          each security key (enroll a backup key too)
        5. Test from a second tty before you rely on it:  sudo -k; sudo true
EOF
fi
