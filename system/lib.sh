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
  sudo -n true 2>/dev/null || sudo -v || die "Well, shit. sudo said no, and this script needs it. Check your password and run it again"
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
# user. scripts/dot-theme shows the ones in ~/.config/kubaos/wallpapers/sets.
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

common_usage() {
  cat <<EOF
Common flags:
  -n, --dry-run   print the commands that change the system, run only checks
  -h, --help      this help
EOF
}
