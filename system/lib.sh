# shellcheck shell=bash disable=SC2034
# Shared helpers for system/*.sh. Sourced, never executed.
#
# Every script accepts --dry-run (print the commands that would change the
# system, run only read-only checks) and --help. Scripts run as your normal
# user and call sudo for the steps that need root. They never reboot.

set -euo pipefail

SYSTEM_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FILES_DIR="$SYSTEM_DIR/files"
DOTFILES_DIR="$(cd "$SYSTEM_DIR/.." && pwd)"
# The KubaOS checkout whose nixpkgs built the curated set (home/curated.nix),
# which the lock screen's PAM modules and the recorder helper must match:
# the root-owned one on an installed machine, else this one (a dev machine
# without it, the test harness).
KUBAOS_DIR="${KUBAOS_DIR:-/usr/share/kubaos}"
[[ -f "$KUBAOS_DIR/flake.nix" ]] || KUBAOS_DIR="$DOTFILES_DIR"

DRY_RUN=0
APT_DIRTY=0
# The user the desktop belongs to. When run as root through sudo, it's the
# caller; otherwise it's us (SUDO_USER may be inherited, e.g. `sudo -iu kuba`).
if ((EUID == 0)) && [[ -n "${SUDO_USER:-}" ]]; then
  TARGET_USER="$SUDO_USER"
else
  TARGET_USER="$(id -un)"
fi

if [[ -t 1 ]]; then
  C_STEP=$'\e[1;34m' C_OK=$'\e[32m' C_WARN=$'\e[1;33m' C_ERR=$'\e[1;31m' C_DRY=$'\e[36m' C_OFF=$'\e[0m'
else
  C_STEP='' C_OK='' C_WARN='' C_ERR='' C_DRY='' C_OFF=''
fi

step() { printf '\n%s==> %s%s\n' "$C_STEP" "$*" "$C_OFF"; }
ok()   { printf '%s  ok%s  %s\n' "$C_OK" "$C_OFF" "$*"; }
info() { printf '      %s\n' "$*"; }
warn() { printf '%s  !!  %s%s\n' "$C_WARN" "$*" "$C_OFF" >&2; }
die()  { printf '%s  xx  %s%s\n' "$C_ERR" "$*" "$C_OFF" >&2; exit 1; }

# run CMD...: execute, or print in dry-run mode.
run() {
  if ((DRY_RUN)); then
    printf '%s  [dry-run]%s' "$C_DRY" "$C_OFF"
    printf ' %q' "$@"
    printf '\n'
  else
    printf '%s  +%s' "$C_DRY" "$C_OFF"
    printf ' %q' "$@"
    printf '\n'
    "$@"
  fi
}

# sudo_run CMD...: run as root (no sudo needed when already root).
sudo_run() {
  if ((EUID == 0)); then run "$@"; else run sudo "$@"; fi
}

# as_root CMD...: read-only command that needs root to look (always executes).
as_root() {
  if ((EUID == 0)); then "$@"; else sudo "$@"; fi
}

require_sudo() {
  ((DRY_RUN)) && return 0
  ((EUID == 0)) && return 0
  command -v sudo >/dev/null || die "sudo is not installed"
  # sudo-rs (Ubuntu 26.04's sudo) rejects `sudo -v` under NOPASSWD, so try a
  # non-interactive no-op first and only prompt when that fails.
  sudo -n true 2>/dev/null || sudo -v || die "Fuck. sudo said no, and this script needs it. Check your password and run it again"
}

# Booted with systemd as PID 1 (false in containers and chroots).
has_systemd() { [[ -d /run/systemd/system ]]; }

in_container() {
  [[ -f /.dockerenv || -f /run/.containerenv ]] && return 0
  command -v systemd-detect-virt >/dev/null && systemd-detect-virt --container -q && return 0
  return 1
}

arch() { dpkg --print-architecture; }
is_amd64() { [[ "$(arch)" == amd64 ]]; }

# Image mode (DOTFILES_IMAGE_BUILD=1, set only by install/image): the
# scripts run in a container whose / becomes the install's payload, not on
# the machine. Units are then enabled, disabled and masked offline (the
# symlinks land in the image), runtime-only steps are skipped, and so is
# anything that needs the target's own disk (the snapper config).
IMAGE_BUILD="${DOTFILES_IMAGE_BUILD:-0}"

# sctl ARGS...: systemctl that is skipped when systemd isn't running. In
# image mode enable/disable/mask still happen, without --now.
sctl() {
  if has_systemd; then
    sudo_run systemctl "$@"
  elif ((IMAGE_BUILD)); then
    case "${1:-}" in
      enable | disable | mask | unmask)
        local a args=()
        for a in "$@"; do [[ "$a" == --now ]] || args+=("$a"); done
        sudo_run systemctl "${args[@]}"
        ;;
      *) info "skip (image build, nothing running): systemctl $*" ;;
    esac
  else
    info "skip (systemd not running here): systemctl $*"
  fi
}

unit_exists() {
  command -v systemctl >/dev/null || return 1
  [[ -n "$(systemctl list-unit-files --no-legend "$1" 2>/dev/null)" ]]
}

pkg_installed() {
  [[ "$(dpkg-query -W -f='${db:Status-Status}' "$1" 2>/dev/null)" == installed ]]
}

pkg_has_candidate() {
  local c
  c="$(apt-cache policy "$1" 2>/dev/null | awk '/Candidate:/ {print $2}')"
  [[ -n "$c" && "$c" != "(none)" ]]
}

apt_update() {
  sudo_run apt-get update
  APT_DIRTY=0
}

apt_update_if_dirty() {
  if ((APT_DIRTY)); then apt_update; fi
}

# apt_install PKG...: install the missing ones with --no-install-recommends.
# Fails early with a clear list if a package has no install candidate.
apt_install() {
  local p missing=() unknown=()
  for p in "$@"; do
    pkg_installed "$p" && continue
    missing+=("$p")
    if ! pkg_has_candidate "$p"; then unknown+=("$p"); fi
  done
  if ((${#missing[@]} == 0)); then
    ok "already installed: $*"
    return 0
  fi
  if ((${#unknown[@]})); then
    if ((DRY_RUN)); then
      warn "no apt candidate yet (fine in dry-run if a repo is added first): ${unknown[*]}"
    else
      die "no apt candidate for: ${unknown[*]} (run apt-get update, check sources)"
    fi
  fi
  if [[ -n "${DEBIAN_FRONTEND:-}" ]]; then
    sudo_run env DEBIAN_FRONTEND="$DEBIAN_FRONTEND" apt-get install -y --no-install-recommends "${missing[@]}"
  else
    sudo_run apt-get install -y --no-install-recommends "${missing[@]}"
  fi
}

apt_purge() {
  local p present=()
  for p in "$@"; do
    if pkg_installed "$p" || [[ "$(dpkg-query -W -f='${db:Status-Status}' "$p" 2>/dev/null)" == config-files ]]; then
      present+=("$p")
    fi
  done
  if ((${#present[@]} == 0)); then
    ok "not installed: $*"
    return 0
  fi
  sudo_run apt-get purge -y --auto-remove "${present[@]}"
}

# same_content A DEST: DEST exists and has A's content. Reads DEST through sudo
# when it isn't readable (netplan files are 0600 root).
same_content() {
  [[ -e "$2" ]] || return 1
  if [[ -r "$2" ]]; then cmp -s "$1" "$2"; else as_root cmp -s "$1" "$2"; fi
}

# install_root_file SRC DEST MODE: copy a repo file into /etc if it differs.
install_root_file() {
  local src="$1" dest="$2" mode="${3:-0644}"
  [[ -f "$src" ]] || die "missing source file $src"
  if same_content "$src" "$dest"; then
    ok "$dest is up to date"
    return 0
  fi
  if [[ -f "$dest" ]]; then info "$dest differs, replacing"; else info "creating $dest"; fi
  sudo_run install -D -m "$mode" "$src" "$dest"
}

# write_root_file DEST MODE <<<CONTENT: like install_root_file, content from stdin.
write_root_file() {
  local dest="$1" mode="${2:-0644}" tmp
  tmp="$(mktemp)"
  cat >"$tmp"
  if same_content "$tmp" "$dest"; then
    ok "$dest is up to date"
    rm -f "$tmp"
    return 0
  fi
  if ((DRY_RUN)); then
    info "would write $dest (mode $mode):"
    sed 's/^/        | /' "$tmp"
  else
    if [[ -f "$dest" ]]; then info "$dest differs, replacing"; else info "creating $dest"; fi
    sudo_run install -D -m "$mode" "$tmp" "$dest"
  fi
  rm -f "$tmp"
}

# render_template SRC KEY=VALUE...: print SRC with @KEY@ replaced.
render_template() {
  local src="$1" content kv
  shift
  shopt -u patsub_replacement 2>/dev/null || true   # bash 5.2: "&" stays literal
  content="$(<"$src")"
  for kv in "$@"; do
    content="${content//@${kv%%=*}@/${kv#*=}}"
  done
  printf '%s\n' "$content"
}

# key_is FILE FINGERPRINT: FILE holds exactly one primary key, with this
# fingerprint (subkeys are fine; extra primary keys are not).
key_is() {
  local fprs
  fprs="$(gpg --show-keys --with-colons "$1" 2>/dev/null |
    awk -F: '$1 == "pub" {want = 1; next} want && $1 == "fpr" {print $10; want = 0}')" || return 1
  [[ "$fprs" == "$2" ]]
}

# fetch_key URL DEST FINGERPRINT: download an OpenPGP key, check its
# fingerprint, store it dearmored at DEST (for apt Signed-By).
fetch_key() {
  local url="$1" dest="$2" fpr="$3" tmp
  if [[ -f "$dest" ]] && key_is "$dest" "$fpr"; then
    ok "key $fpr present at $dest"
    return 0
  fi
  if ((DRY_RUN)); then
    run curl -fsSL "$url" -o "<tmp>"
    info "would verify fingerprint $fpr, then: gpg --dearmor > $dest"
    return 0
  fi
  tmp="$(mktemp -d)"
  curl -fsSL "$url" -o "$tmp/key"
  if ! key_is "$tmp/key" "$fpr"; then
    rm -rf "$tmp"
    die "key from $url is not (only) $fpr, refusing to trust it"
  fi
  gpg --dearmor <"$tmp/key" >"$tmp/key.gpg" 2>/dev/null || cp "$tmp/key" "$tmp/key.gpg"
  sudo_run install -D -m 0644 "$tmp/key.gpg" "$dest"
  rm -rf "$tmp"
}

# add_apt_source NAME CONTENT: write /etc/apt/sources.list.d/NAME.sources.
add_apt_source() {
  local name="$1" dest="/etc/apt/sources.list.d/$1.sources"
  if [[ -f "$dest" ]] && [[ "$(<"$dest")" == "$2" ]]; then
    ok "apt source $name configured"
    return 0
  fi
  printf '%s\n' "$2" | write_root_file "$dest" 0644
  APT_DIRTY=1
}

snapper_ready() {
  command -v snapper >/dev/null && [[ -f /etc/snapper/configs/root ]]
}

# snap_create DESCRIPTION: one root snapshot per description (skips repeats).
snap_create() {
  local desc="$1"
  if ! snapper_ready; then
    info "snapper not configured, skipping snapshot \"$desc\""
    return 0
  fi
  # grep without -q reads all input: with pipefail, -q exiting early can kill
  # snapper with SIGPIPE and turn a match into a failure.
  if as_root snapper -c root list --columns description 2>/dev/null | grep -F -- "$desc" >/dev/null; then
    ok "snapshot \"$desc\" already exists"
    return 0
  fi
  sudo_run snapper -c root create --cleanup-algorithm number --description "$desc"
}

user_home() { getent passwd "$1" | cut -d: -f6; }

# PAM modules for the Quickshell lock screen. Quickshell links Nix's libpam,
# which can't load Ubuntu's modules (its loader doesn't search Ubuntu's library
# dirs), so the lock's PAM files load nixpkgs' builds through these links. The
# links and the GC roots live in root-owned dirs, so nothing in $HOME decides
# what code PAM loads.
#   pam_fprintd.so  /etc/pam.d/qs-lock-fingerprint (system/50-hyprland.sh)
#   pam_u2f.so      /etc/pam.d/qs-lock-u2f         (system/70-bitwarden.sh)
PAM_FPRINTD_LINK=/usr/local/lib/security/pam_fprintd.so
PAM_FPRINTD_GCROOT=/nix/var/nix/gcroots/dotfiles-pam-fprintd
PAM_U2F_LINK=/usr/local/lib/security/pam_u2f.so
PAM_U2F_GCROOT=/nix/var/nix/gcroots/dotfiles-pam-u2f

# link_nix_pam_module ATTR MODULE LINK GCROOT: build nixpkgs#ATTR from
# KubaOS's pinned nixpkgs (KUBAOS_DIR: the same nixpkgs Quickshell and its
# libpam come from), pin it with GCROOT and point LINK at its
# lib/security/MODULE. Re-run after every KubaOS update.
link_nix_pam_module() {
  local attr="$1" module="$2" link="$3" gcroot="$4" out so
  if ((DRY_RUN)); then
    run nix build --no-link --print-out-paths --inputs-from "$KUBAOS_DIR" "nixpkgs#$attr"
    sudo_run ln -sfn "<$attr store path>" "$gcroot"
    sudo_run install -d -m 0755 "$(dirname "$link")"
    sudo_run ln -sfn "<$attr store path>/lib/security/$module" "$link"
    return 0
  fi
  out="$(nix build --no-link --print-out-paths --inputs-from "$KUBAOS_DIR" "nixpkgs#$attr")" ||
    die "nix build nixpkgs#$attr failed"
  so="$out/lib/security/$module"
  [[ -f "$so" ]] || die "no $module in $out"
  if [[ "$(readlink "$gcroot" 2>/dev/null)" == "$out" &&
    "$(readlink "$link" 2>/dev/null)" == "$so" ]]; then
    ok "$link -> $so"
    return 0
  fi
  # GC root first, so the store path can't be collected while the PAM link points at it.
  sudo_run ln -sfn "$out" "$gcroot"
  sudo_run install -d -m 0755 "$(dirname "$link")"
  sudo_run ln -sfn "$so" "$link"
}

link_pam_fprintd() { link_nix_pam_module fprintd pam_fprintd.so "$PAM_FPRINTD_LINK" "$PAM_FPRINTD_GCROOT"; }
link_pam_u2f() { link_nix_pam_module pam_u2f pam_u2f.so "$PAM_U2F_LINK" "$PAM_U2F_GCROOT"; }

# The wallpaper sets (flake input kubaos-wallpapers), read-only for every
# user. scripts/dot-theme shows the ones in ~/.config/kubaos/desktop/wallpapers/sets.
WALLPAPERS_LINK=/usr/share/kubaos/wallpapers
WALLPAPERS_GCROOT=/nix/var/nix/gcroots/dotfiles-wallpapers

# link_wallpapers: point WALLPAPERS_LINK at the flake's pinned wallpaper
# input, pinned with WALLPAPERS_GCROOT (the payload build ships every
# dotfiles-* root). From KUBAOS_DIR's lock, so a new set comes with a release.
# The link sits inside the /usr/share/kubaos clone (.gitignore: /wallpapers).
# Re-run after every flake update.
link_wallpapers() {
  local out
  if ((DRY_RUN)); then
    run nix eval --raw "$KUBAOS_DIR#lib.wallpapers"
    sudo_run ln -sfn "<wallpapers store path>" "$WALLPAPERS_GCROOT"
    sudo_run install -d -m 0755 "$(dirname "$WALLPAPERS_LINK")"
    sudo_run ln -sfn "<wallpapers store path>" "$WALLPAPERS_LINK"
    return 0
  fi
  out="$(nix eval --raw "$KUBAOS_DIR#lib.wallpapers")" || die "nix eval .#lib.wallpapers failed"
  [[ -d "$out" ]] || die "no wallpapers at $out"
  if [[ "$(readlink "$WALLPAPERS_GCROOT" 2>/dev/null)" == "$out" &&
    "$(readlink "$WALLPAPERS_LINK" 2>/dev/null)" == "$out" ]]; then
    ok "$WALLPAPERS_LINK -> $out"
    return 0
  fi
  sudo_run ln -sfn "$out" "$WALLPAPERS_GCROOT"
  sudo_run install -d -m 0755 "$(dirname "$WALLPAPERS_LINK")"
  sudo_run ln -sfn "$out" "$WALLPAPERS_LINK"
}

# Packages held with the Hyprland stack (50-hyprland.sh; kubaos update holds the same set): the
# compositor's own packages plus every hypr*/aquamarine library.
HYPR_HOLD_RE='^(hypr|libhypr|libaquamarine|xdg-desktop-portal-hyprland)'

# hypr_installed: installed packages matching HYPR_HOLD_RE.
hypr_installed() {
  dpkg-query -W -f='${db:Status-Status} ${Package}\n' 2>/dev/null |
    awk -v re="$HYPR_HOLD_RE" '$1 == "installed" && $2 ~ re {print $2}'
}

# install_keyboard_writer: what `kubaos keyboard` writes a new default layout
# for the console and the disk unlock with. The root helper that writes
# /etc/default/keyboard, the polkit action pkexec runs it
# under, and the rule that lets an admin at the machine through without a
# password. Ubuntu's localed refuses SetX11Keyboard through a D-Bus policy and
# no longer writes that file (files/keyboard/kubaos-keyboard-write says more).
# 50-hyprland.sh runs it, and a migration on machines installed before it.
install_keyboard_writer() {
  install_root_file "$FILES_DIR/keyboard/kubaos-keyboard-write" /usr/local/libexec/kubaos-keyboard-write 0755
  install_root_file "$FILES_DIR/polkit/org.kubaos.keyboard.policy" /usr/share/polkit-1/actions/org.kubaos.keyboard.policy 0644
  install_root_file "$FILES_DIR/polkit/50-kubaos-keyboard.rules" /etc/polkit-1/rules.d/50-kubaos-keyboard.rules 0644
}

# install_power_helper [--actions]: what the power stops (crates/kubaos-power)
# need as root. The helper that writes their knobs in /sys, the polkit action
# pkexec runs it under and the rule that lets an admin at the machine through
# without a password; and amdgpu's FreeSync video modes, so the laptop panel
# drops to 60 Hz without a modeset. modprobe.d reaches the driver through the
# initramfs, so while an installed kernel's initramfs lacks the file they
# are all rebuilt; one that failed to build is rebuilt on the next run. With --actions (the first
# install, and the migration), power-profiles-daemon's GPU action goes on, so
# its power-saver profile runs the GPU low for every tool that switches it,
# and its panel action off: that one rewrites the panel's power saving at
# every battery percent, and the stops own that knob. Without it the actions
# stay as the user left them. 30-hardware.sh runs it, and a migration on
# machines installed before it.
install_power_helper() {
  local actions=0 initrd modules
  [[ ${1:-} == --actions || ! -e /usr/local/libexec/kubaos-power-write ]] && actions=1
  install_root_file "$FILES_DIR/power/kubaos-power-write" /usr/local/libexec/kubaos-power-write 0755
  install_root_file "$FILES_DIR/polkit/org.kubaos.power.policy" /usr/share/polkit-1/actions/org.kubaos.power.policy 0644
  install_root_file "$FILES_DIR/polkit/50-kubaos-power.rules" /etc/polkit-1/rules.d/50-kubaos-power.rules 0644
  install_root_file "$FILES_DIR/modprobe.d/kubaos-amdgpu.conf" /etc/modprobe.d/kubaos-amdgpu.conf 0644
  if ! in_container && command -v update-initramfs >/dev/null && command -v lsinitramfs >/dev/null; then
    # Only installed kernels: an initramfs left behind by a removed one is
    # never rebuilt, and would ask for a rebuild at every run.
    for modules in /lib/modules/*; do
      initrd="/boot/initrd.img-${modules##*/}"
      [[ -d $modules && -e $initrd ]] || continue
      if ! as_root lsinitramfs "$initrd" 2>/dev/null | grep -q 'kubaos-amdgpu\.conf$'; then
        info "rebuilding the initramfs for amdgpu's FreeSync video modes (takes effect at the next boot)"
        sudo_run update-initramfs -u -k all
        break
      fi
    done
  fi
  if ((actions)) && command -v powerprofilesctl >/dev/null && has_systemd; then
    if powerprofilesctl list-actions 2>/dev/null | grep -q '^Name: amdgpu_dpm$'; then
      sudo_run powerprofilesctl configure-action amdgpu_dpm --enable
    fi
    if powerprofilesctl list-actions 2>/dev/null | grep -q '^Name: amdgpu_panel_power$'; then
      sudo_run powerprofilesctl configure-action amdgpu_panel_power --disable
    fi
  fi
}

# install_proxy_helper: what the Proxy (crates/kubaos-proxy) needs as root.
# The helper that puts the work network's system-wide parts in place (apt's,
# Docker's, Nix's and Chrome's proxy, the company CA, the kill switch), and
# the same script as kubaos-proxy-pin, which pins the proxy and the CA the
# first may put in place; the polkit actions pkexec runs them under, and the
# rule that lets an admin at the machine apply what is pinned without a
# password (pinning always asks for it). 20-base.sh runs it, and a migration
# on machines installed before it.
install_proxy_helper() {
  install_root_file "$FILES_DIR/proxy/kubaos-proxy-write" /usr/local/libexec/kubaos-proxy-write 0755
  install_root_file "$FILES_DIR/proxy/kubaos-proxy-write" /usr/local/libexec/kubaos-proxy-pin 0755
  install_root_file "$FILES_DIR/polkit/org.kubaos.proxy.policy" /usr/share/polkit-1/actions/org.kubaos.proxy.policy 0644
  install_root_file "$FILES_DIR/polkit/50-kubaos-proxy.rules" /etc/polkit-1/rules.d/50-kubaos-proxy.rules 0644
}

# install_scx: the CPU scheduler the power stops can run (crates/kubaos-power,
# `scheduler = "lavd"` in power.toml). scx_lavd from KUBAOS_DIR's curated set
# (`.#scx-lavd`, home/curated.nix), pinned with SCX_GCROOT and linked where
# kubaos-scx.service runs it as root, and the unit itself. It never enables or
# starts the unit: LAVD is off until the user turns it on, and then the power
# stops' root helper does. A LAVD running on an older build or unit file is
# restarted on the new one. 60-nix.sh and relink.sh (every update and rollback) run it.
SCX_LINK=/usr/local/libexec/kubaos-scx-lavd
SCX_GCROOT=/nix/var/nix/gcroots/dotfiles-scx-lavd
install_scx() {
  local out bin changed=0
  same_content "$FILES_DIR/systemd/kubaos-scx.service" /etc/systemd/system/kubaos-scx.service || changed=1
  install_root_file "$FILES_DIR/systemd/kubaos-scx.service" /etc/systemd/system/kubaos-scx.service 0644
  if ((changed)) && has_systemd; then
    sudo_run systemctl daemon-reload
  fi
  if ((DRY_RUN)); then
    run nix build --no-link --print-out-paths "$KUBAOS_DIR#scx-lavd"
    sudo_run ln -sfn "<scx-lavd store path>" "$SCX_GCROOT"
    sudo_run ln -sfn "<scx-lavd store path>/bin/scx_lavd" "$SCX_LINK"
    return 0
  fi
  out="$(nix build --no-link --print-out-paths "$KUBAOS_DIR#scx-lavd")" || die "nix build .#scx-lavd failed"
  bin="$out/bin/scx_lavd"
  [[ -x "$bin" ]] || die "no scx_lavd in $out"
  if [[ "$(readlink "$SCX_GCROOT" 2>/dev/null)" == "$out" && "$(readlink "$SCX_LINK" 2>/dev/null)" == "$bin" ]]; then
    ok "$SCX_LINK -> $bin"
  else
    # GC root first, so the store path can't be collected while the link points at it.
    sudo_run ln -sfn "$out" "$SCX_GCROOT"
    sudo_run install -d -m 0755 "$(dirname "$SCX_LINK")"
    sudo_run ln -sfn "$bin" "$SCX_LINK"
    changed=1
  fi
  if ((changed)) && has_systemd && systemctl is-active --quiet kubaos-scx.service; then
    # HAZARD: flush the start counter first, as the root helper does. A LAVD
    # that crashed and came back within StartLimitIntervalSec would otherwise
    # hit StartLimitBurst on this restart and be left failed.
    sudo_run systemctl reset-failed kubaos-scx.service
    sudo_run systemctl restart kubaos-scx.service
  fi
}

# install_mac_accents: the Mac-style accents on Left Alt. After Left Alt + e,
# `, u, i or n, keyd sends a dead key on a spare keycode and then the next
# letter (files/keyd/kubaos.conf), the `kubaos:mac-accents` XKB option gives
# those keycodes their symbols in every layout (files/xkb, which libxkbcommon
# reads from /etc/xkb), and a polkit rule lets
# the desktop user start and stop keyd, which `kubaos keyboard accents` does.
# 50-hyprland.sh runs it, and a migration on machines installed before it.
install_mac_accents() {
  local changed=0
  install_root_file "$FILES_DIR/xkb/rules/evdev" /etc/xkb/rules/evdev 0644
  install_root_file "$FILES_DIR/xkb/symbols/kubaos" /etc/xkb/symbols/kubaos 0644
  install_root_file "$FILES_DIR/polkit/50-kubaos-keyd.rules" /etc/polkit-1/rules.d/50-kubaos-keyd.rules 0644
  same_content "$FILES_DIR/keyd/kubaos.conf" /etc/keyd/kubaos.conf || changed=1
  # The config goes in first, so keyd starts with it when the package starts it.
  install_root_file "$FILES_DIR/keyd/kubaos.conf" /etc/keyd/kubaos.conf 0644
  group_install accents
  sctl enable keyd.service
  # try-restart: a keyd the user stopped (accents off) stays stopped.
  if ((changed)) && has_systemd; then
    sudo_run systemctl try-restart keyd.service
  fi
}

# boot_unit UNIT enable|disable|mask: whether UNIT starts at boot, when it
# exists. Nothing starts or stops now, so it works offline too (image
# builds, the test container); the change shows at the next boot.
boot_unit() {
  local unit="$1" want="$2" state
  unit_exists "$unit" || return 0
  state="$(systemctl is-enabled "$unit" 2>/dev/null || true)"
  case "$want:$state" in
    enable:enabled | disable:disabled | disable:masked | mask:masked)
      ok "$unit is $state"
      return 0
      ;;
  esac
  sudo_run systemctl "$want" "$unit"
}

# docker_on_demand: Docker starts on its first use, through docker.socket,
# not at boot. docker.service wants network-online.target, so starting it at
# boot puts the wait for the network on the way to the desktop. containerd
# starts with it (docker.service wants it). Containers with a restart policy
# come back on the first `docker` command, not at boot.
docker_on_demand() {
  unit_exists docker.service || return 0
  install_root_file "$FILES_DIR/systemd/docker.service.d/kubaos-stop.conf" \
    /etc/systemd/system/docker.service.d/kubaos-stop.conf 0644
  boot_unit docker.socket enable
  boot_unit docker.service disable
  boot_unit containerd.service disable
  # A Docker running now keeps running; one that isn't answers on the socket
  # from now on, not from the next boot.
  if has_systemd && ! systemctl is-active -q docker.socket; then
    sudo_run systemctl start docker.socket
  fi
}

# install_fast_shutdown_boot: what keeps shutdown and boot short. The system
# and user managers kill a unit 10 s after SIGTERM, not 90 s; Docker starts on
# demand; NetworkManager-wait-online is off, since nothing on a laptop's way
# to the desktop needs the network up; Ubuntu's console palette (setvtrgb) is
# masked, as tty1 shows nothing but Plymouth and Hyprland. 20-base.sh runs
# it, and a migration on machines installed before it.
install_fast_shutdown_boot() {
  local reload=0
  same_content "$FILES_DIR/systemd/system.conf.d/kubaos-timeouts.conf" \
    /etc/systemd/system.conf.d/kubaos-timeouts.conf || reload=1
  install_root_file "$FILES_DIR/systemd/system.conf.d/kubaos-timeouts.conf" \
    /etc/systemd/system.conf.d/kubaos-timeouts.conf 0644
  install_root_file "$FILES_DIR/systemd/user.conf.d/kubaos-timeouts.conf" \
    /etc/systemd/user.conf.d/kubaos-timeouts.conf 0644
  docker_on_demand
  boot_unit NetworkManager-wait-online.service disable
  boot_unit setvtrgb.service mask
  # A daemon-reload rereads system.conf.d too, so the next shutdown is already
  # a short one. User managers read user.conf.d when they start, at login.
  if ((reload)) && has_systemd; then
    sudo_run systemctl daemon-reload
  fi
}

# file_locked FILE: whether a process holds a lock on FILE, from the kernel's lock table. It is
# read-only and sees flock and fcntl locks alike; unattended-upgrade takes an fcntl one, which
# `flock -n` on the file would not notice.
file_locked() {
  local inode
  [[ -e "$1" ]] || return 1
  inode="$(stat -c %i -- "$1")"
  grep -q ":$inode " /proc/locks
}

# retire_ubuntu_updaters: nothing but `kubaos update` runs apt. unattended-upgrades and the
# apt-daily timers refresh lists and install packages in the background, where they can hold the
# dpkg lock while an update runs, wake the machine and use the network and battery. Their timers
# and unattended-upgrades.service stop now; the units are disabled and masked, so a package
# upgrade can't enable them again; apt's periodic jobs are set to 0.
#
# Stopping unattended-upgrades.service sends SIGTERM to an upgrade that is running, which stops
# it gracefully, and the stop blocks until it has, up to 30 minutes. So the stop is skipped while
# /var/run/unattended-upgrades.lock is held; the disable and mask still apply, and it does not
# start again. apt-daily's services are never stopped: a run in progress finishes.
# 10-strip.sh runs it, and a migration on machines installed before it.
retire_ubuntu_updaters() {
  local unit
  install_root_file "$FILES_DIR/apt/apt.conf.d/99kubaos-no-periodic" \
    /etc/apt/apt.conf.d/99kubaos-no-periodic 0644
  for unit in apt-daily.timer apt-daily-upgrade.timer unattended-upgrades.service; do
    has_systemd && systemctl is-active -q "$unit" || continue
    if [[ "$unit" == unattended-upgrades.service ]] && file_locked /var/run/unattended-upgrades.lock; then
      info "an unattended upgrade is running: $unit is only disabled and masked, not stopped"
    else
      sudo_run systemctl stop "$unit"
    fi
  done
  for unit in apt-daily.timer apt-daily-upgrade.timer unattended-upgrades.service \
    apt-daily.service apt-daily-upgrade.service; do
    boot_unit "$unit" disable
    boot_unit "$unit" mask
  done
}

# install_shutdown_splash: the splash on poweroff and reboot shows every time
# and gets to play out. Plymouth's shutdown units start once the sessions
# have ended, so Hyprland no longer holds the screen, and
# kubaos-shutdown-splash.service holds the final poweroff for the length of
# the animation (both explained in their files). 25-boot.sh runs it, and a
# migration on machines installed before it.
install_shutdown_splash() {
  local unit changed=0 dest
  for unit in poweroff reboot halt kexec; do
    dest="/etc/systemd/system/plymouth-$unit.service.d/kubaos-after-session.conf"
    same_content "$FILES_DIR/systemd/plymouth-shutdown.d/kubaos-after-session.conf" "$dest" || changed=1
    install_root_file "$FILES_DIR/systemd/plymouth-shutdown.d/kubaos-after-session.conf" "$dest" 0644
  done
  same_content "$FILES_DIR/systemd/kubaos-shutdown-splash.service" \
    /etc/systemd/system/kubaos-shutdown-splash.service || changed=1
  install_root_file "$FILES_DIR/systemd/kubaos-shutdown-splash.service" \
    /etc/systemd/system/kubaos-shutdown-splash.service 0644
  # The running manager reads unit files and their wants at a reload; without
  # one the next shutdown still runs on the old ones.
  if ((changed)) && has_systemd; then
    sudo_run systemctl daemon-reload
  fi
  boot_unit kubaos-shutdown-splash.service enable
}

PLY_THEME_DIR=/usr/share/plymouth/themes/kubaos
GRUB_THEME_DIR=/boot/grub/themes/kubaos

# install_dir SRC DEST: install_root_file for every file directly in SRC.
install_dir() {
  local f
  for f in "$1"/*; do
    [[ -f "$f" ]] || continue
    install_root_file "$f" "$2/$(basename "$f")" "$(if [[ -x "$f" ]]; then echo 0755; else echo 0644; fi)"
  done
}

# grub_theme_fonts THEME: the fonts THEME (a theme.txt) names, "<family> Regular <size>", one
# `TTF|FAMILY|SIZE|PF2` line each, PF2 the .pf2 file's name without its extension. Dies on a
# family no font in system/files/fonts carries.
grub_theme_fonts() {
  local -a names
  local line family size ttf
  mapfile -t names < <(grep -oE '"[A-Za-z ]+ Regular [0-9]+"' "$1" | tr -d '"' | sort -u)
  for line in "${names[@]}"; do
    size="${line##* }"
    family="${line% Regular *}"
    case "$family" in
      Geist) ttf=Geist-Regular.ttf ;;
      "Geist Medium") ttf=Geist-Medium.ttf ;;
      "Geist Mono") ttf=GeistMono-Regular.ttf ;;
      *) die "$1 names the font $line, and no font in $FILES_DIR/fonts is $family" ;;
    esac
    printf '%s|%s|%s|%s\n' "$ttf" "$family" "$size" "$(tr 'A-Z' 'a-z' <<<"${ttf%.ttf}")-$size"
  done
}

# install_boot_theme: the Plymouth theme (the splash and the disk passphrase prompt) with the
# fonts its label plugin and the initramfs hook use, and the GRUB theme with the .pf2 fonts its
# theme.txt names. Fonts neither theme uses any more go: Ubuntu's 00_header loads every .pf2 in
# the theme's directory on every boot, and the plymouth hook copies the whole theme into every
# initramfs. Sets BOOT_THEME_CHANGED=1 when a file of either changed. 25-boot.sh runs it, and a
# migration on a machine whose theme is older.
#
# HAZARD: install the fonts before the hook. The hook links Plymouth-monospace.ttf to the theme's
# Geist Mono, and plymouthd crashes at the passphrase prompt without a monospace font.
install_boot_theme() {
  local f before after specs ttf family size name dest src
  local -a wanted=()
  BOOT_THEME_CHANGED=0
  # Read before anything is installed: a font theme.txt names that no file is stops here.
  specs="$(grub_theme_fonts "$FILES_DIR/grub/kubaos/theme.txt")"
  before="$(boot_theme_state)"
  install_dir "$FILES_DIR/plymouth/kubaos" "$PLY_THEME_DIR"
  for f in Geist-Regular.ttf Geist-OFL.txt GeistMono-Regular.ttf; do
    install_root_file "$FILES_DIR/fonts/$f" "$PLY_THEME_DIR/$f" 0644
  done
  install_root_file "$FILES_DIR/plymouth/conf/plymouthd.conf" /etc/plymouth/plymouthd.conf 0644
  install_root_file "$FILES_DIR/initramfs-tools/hooks/kubaos-plymouth" /etc/initramfs-tools/hooks/kubaos-plymouth 0755
  # The monospace font of an older release, once Geist Mono is there to take its place.
  if [[ -f $PLY_THEME_DIR/GeistMono-Regular.ttf ]] || ((DRY_RUN)); then
    for f in JetBrainsMono-Regular.ttf JetBrainsMono-OFL.txt; do
      if [[ -e $PLY_THEME_DIR/$f ]]; then
        sudo_run rm -f "$PLY_THEME_DIR/$f"
      fi
    done
  fi

  install_dir "$FILES_DIR/grub/kubaos" "$GRUB_THEME_DIR"
  # Fonts: ASCII and Latin-1 only, so GRUB loads them fast on every boot. theme.txt names them
  # "<family> <style> <size>"; -n sets the family, because Geist-Medium.ttf would otherwise also
  # call itself "Geist Regular".
  while IFS='|' read -r ttf family size name; do
    wanted+=("$name.pf2")
    dest="$GRUB_THEME_DIR/$name.pf2"
    src="$FILES_DIR/fonts/$ttf"
    # Rebuilt when missing or older than its TTF.
    if [[ -f "$dest" && ! "$src" -nt "$dest" ]]; then
      ok "$dest"
    else
      # grub-mkfont warns about every OpenType feature it doesn't use.
      sudo_run grub-mkfont -n "$family" -s "$size" -r 0x20-0x7e,0xa0-0xff -o "$dest" "$src" \
        2> >(grep -v 'unsupported font feature' >&2 || true)
    fi
  done <<<"$specs"
  for f in "$GRUB_THEME_DIR"/*.pf2; do
    if [[ -e $f && " ${wanted[*]} " != *" $(basename "$f") "* ]]; then
      sudo_run rm -f "$f"
    fi
  done
  after="$(boot_theme_state)"
  [[ "$before" == "$after" ]] || BOOT_THEME_CHANGED=1
}

# boot_theme_state: a line per file of the installed boot themes and the hook, with its size and
# time, to tell whether install_boot_theme changed any.
boot_theme_state() {
  as_root find "$PLY_THEME_DIR" "$GRUB_THEME_DIR" /etc/initramfs-tools/hooks/kubaos-plymouth \
    /etc/plymouth/plymouthd.conf -maxdepth 1 -type f -printf '%p %s %T@\n' 2>/dev/null | sort || true
}

# nix_parallel_builds: the Nix daemon runs as many builds at once as the
# machine has cores (`max-jobs = auto` in /etc/nix/nix.conf), where Nix's
# default is one. A release rebuilds KubaOS's crates, nine small builds that
# one at a time leave most cores idle. `cores` keeps its default, every core
# for each build: the builds that need them (Kōbō's dependencies, the crates
# that compile kubaos-core) are the longest, and the rest finish in seconds.
# On 24 cores, capping it at 8 made a release's crates 11% slower (60 s
# against 54 s) and a Cargo.lock change 10% slower (224 s against 203 s). A
# max-jobs line already in the file is the user's and stays. 60-nix.sh runs
# it, and a migration on machines installed before it.
nix_parallel_builds() {
  local conf=/etc/nix/nix.conf
  if [[ -f $conf ]] && grep -Eq '^[[:space:]]*max-jobs[[:space:]]*=' "$conf"; then
    ok "$conf sets max-jobs already"
    return 0
  fi
  if ((DRY_RUN)); then
    info "would append to $conf: max-jobs = auto"
  else
    sudo_run sh -c 'printf "\n# KubaOS: builds side by side (nix_parallel_builds, system/lib.sh)\nmax-jobs = auto\n" >> /etc/nix/nix.conf'
  fi
  # The daemon reads nix.conf when it starts. Its unit kills only the main
  # process, so builds already running finish.
  if unit_exists nix-daemon.service && has_systemd; then
    sctl restart nix-daemon.service
  fi
}

# nix_conf_has CONF SETTING VALUE: CONF sets SETTING, or its `extra-` form, to
# a list that holds VALUE.
nix_conf_has() {
  local conf="$1" setting="$2" value="$3"
  [[ -f $conf ]] || return 1
  # HAZARD: not `grep -q`. It exits at the first match, and the first grep's
  # SIGPIPE then fails the pipeline under pipefail, as if nothing matched.
  grep -E "^[[:space:]]*(extra-)?${setting}[[:space:]]*=" "$conf" | grep -wF -- "$value" >/dev/null
}

# KubaOS's binary cache on Cachix. CI pushes the curated set's own builds to
# it from every push to main (.github/workflows/cache.yml).
KUBAOS_CACHE_URL="https://kuba-os.cachix.org"
KUBAOS_CACHE_KEY="kuba-os.cachix.org-1:BAhDPs8FW+W0SaOzsx7yS9sZmcb6ZjdLc09hBeKXIE0="

# nix_binary_cache: the Nix daemon downloads KubaOS's packages from its
# binary cache instead of building them (`extra-substituters` and
# `extra-trusted-public-keys` in /etc/nix/nix.conf). A path the cache doesn't
# have, such as a release CI is still building, builds on the machine as
# before. The `extra-` settings add to Nix's own (cache.nixos.org) and to any
# the user set, so the file's other lines stay; each is written only when the
# file doesn't name the cache or key already. The daemon takes substituters
# from this file as they are, so no user needs to be in `trusted-users` (as
# `cachix use` would want): a trusted user is effectively root. 60-nix.sh runs
# it, and a migration on machines installed before it.
nix_binary_cache() {
  local conf=/etc/nix/nix.conf lines=""
  nix_conf_has "$conf" substituters "$KUBAOS_CACHE_URL" ||
    lines+="extra-substituters = $KUBAOS_CACHE_URL\n"
  nix_conf_has "$conf" trusted-public-keys "$KUBAOS_CACHE_KEY" ||
    lines+="extra-trusted-public-keys = $KUBAOS_CACHE_KEY\n"
  if [[ -z $lines ]]; then
    ok "$conf uses $KUBAOS_CACHE_URL already"
    return 0
  fi
  if ((DRY_RUN)); then
    lines="${lines%\\n}"
    info "would append to $conf: ${lines//\\n/; }"
  else
    # shellcheck disable=SC2016 # $1 and $2 are the inner shell's arguments.
    sudo_run sh -c 'printf "\n# KubaOS: its binary cache (nix_binary_cache, system/lib.sh)\n%b" "$1" >>"$2"' \
      sh "$lines" "$conf"
  fi
  # The daemon reads nix.conf when it starts. Its unit kills only the main
  # process, so builds already running finish.
  if unit_exists nix-daemon.service && has_systemd; then
    sctl restart nix-daemon.service
  fi
}

# shellcheck source=system/manifest.sh
source "$SYSTEM_DIR/manifest.sh"

common_usage() {
  cat <<EOF
Common flags:
  -n, --dry-run   print the commands that change the system, run only checks
  -h, --help      this help
EOF
}
