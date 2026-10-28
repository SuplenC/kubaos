# shellcheck shell=bash disable=SC2034
# In-container helpers for the test harness. Sourced by test/lib/entry, runs
# as root inside the dotfiles-test image. The repo under test is /src
# (read-only); it's copied to ~/Projects/dotfiles for each run.

set -euo pipefail

SRC=/src
USER_NAME=kuba
USER_HOME=/home/$USER_NAME
REPO=$USER_HOME/Projects/dotfiles
RUNTIME_DIR=/run/user/1000
CACHE=/var/cache/dotfiles-test
LOGS=/tmp/dotfiles-test
HM_CONFIG="$USER_NAME@vm"
NIX_BIN=/nix/var/nix/profiles/default/bin
GCROOTS=/nix/var/nix/gcroots/dotfiles-test
mkdir -p "$LOGS"
chmod 0777 "$LOGS"

if [[ -t 1 ]]; then
  C_B=$'\e[1m' C_OK=$'\e[32m' C_ERR=$'\e[31m' C_DIM=$'\e[2m' C_OFF=$'\e[0m'
else
  C_B='' C_OK='' C_ERR='' C_DIM='' C_OFF=''
fi
say() { printf '%s==> %s%s\n' "$C_B" "$*" "$C_OFF"; }
note() { printf '%s    %s%s\n' "$C_DIM" "$*" "$C_OFF"; }
die() { printf '%sxx  %s%s\n' "$C_ERR" "$*" "$C_OFF" >&2; exit 1; }
now() { date +%s.%N; }
since() { awk -v a="$1" -v b="$(now)" 'BEGIN { printf "%.1f", b - a }'; }
# peak_memory: the container's highest memory use so far, for sizing
# DOTFILES_TEST_MEM.
peak_memory() {
  local f=/sys/fs/cgroup/memory.peak
  [[ -r $f ]] && awk '{ printf "%.1f GB", $1 / 1e9 }' "$f" || echo unknown
}

# The environment login(1) would give kuba on a tty, plus a runtime dir.
JENV=(
  HOME="$USER_HOME" USER="$USER_NAME" LOGNAME="$USER_NAME" SHELL=/usr/bin/zsh
  TERM="${TERM:-xterm-256color}" LANG=C.UTF-8 TZ=Etc/UTC
  PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/games:/usr/games
  XDG_RUNTIME_DIR="$RUNTIME_DIR" DBUS_SESSION_BUS_ADDRESS="unix:path=$RUNTIME_DIR/bus"
)
# as_user CMD...: run as kuba with a clean login-like environment.
as_user() { runuser -u "$USER_NAME" -- env -i "${JENV[@]}" "$@"; }
# zlogin CMD: CMD in kuba's login + interactive zsh (what a terminal gets).
zlogin() { as_user zsh -l -i -c "$1"; }
# zrun CMD: CMD in a login zsh (PATH and session vars, no interactive init).
zrun() { as_user zsh -l -c "$1"; }

# --- once per container -------------------------------------------------------
# start_nix_daemon: the daemon, with bounds for its builds, which share the
# machine with its session and other work (a Quickshell or crate build takes
# gigabytes per core), and the machine's own store as the first substituter
# when test/run mounted it, so nothing the machine has is built or downloaded
# again. That store is mounted read-only, hence read-only=true (no locks), and
# its paths carry no signature, hence trusted. They are the daemon's options,
# not nix.conf lines, so nix.conf stays what system/60-nix.sh writes.
start_nix_daemon() {
  local opts=(--option max-jobs "${DOTFILES_TEST_NIX_JOBS:-2}" --option cores "${DOTFILES_TEST_NIX_CORES:-4}")
  if [[ -d /host/nix/store && -f /host/nix/var/nix/db/db.sqlite ]]; then
    opts+=(--option extra-experimental-features read-only-local-store
      --option extra-substituters 'local?root=/host&read-only=true&trusted=true&priority=10')
  fi
  [[ -S /nix/var/nix/daemon-socket/socket ]] && rm -f /nix/var/nix/daemon-socket/socket
  if ! pgrep -x nix-daemon >/dev/null; then
    "$NIX_BIN/nix-daemon" "${opts[@]}" >"$LOGS/nix-daemon.log" 2>&1 &
  fi
  for _ in $(seq 50); do
    [[ -S /nix/var/nix/daemon-socket/socket ]] && return 0
    sleep 0.1
  done
  die "nix-daemon didn't start ($LOGS/nix-daemon.log)"
}

# What boot does on the real machine: tmpfiles (the lock screen's unix_chkpwd
# link, /run/opengl-driver once 60-nix.sh wrote its conf) and the user's
# runtime dir (logind).
boot_like() {
  if [[ -f /etc/tmpfiles.d/qs-lock-unix-chkpwd.conf ]]; then
    systemd-tmpfiles --create /etc/tmpfiles.d/qs-lock-unix-chkpwd.conf >/dev/null 2>&1 ||
      { mkdir -p /run/wrappers/bin && ln -sfn /usr/sbin/unix_chkpwd /run/wrappers/bin/unix_chkpwd; }
  fi
  install -d -o "$USER_NAME" -g "$USER_NAME" -m 0700 "$RUNTIME_DIR"
  # The user bus systemd --user (dbus-user-session) provides on a real login;
  # home-manager's dconf step needs it.
  if [[ ! -S $RUNTIME_DIR/bus ]]; then
    runuser -u "$USER_NAME" -- dbus-daemon --session --fork --address="unix:path=$RUNTIME_DIR/bus" >/dev/null
  fi
  mkdir -p /run/dbus
  if [[ ! -S /run/dbus/system_bus_socket ]]; then
    dbus-daemon --system --fork >/dev/null 2>&1 || true
  fi
}

# Local flake inputs (test/run mounts them at their host path, read-only,
# owned by root here): git refuses repositories owned by someone else unless
# they're listed as safe. The container is thrown away after the run.
trust_local_inputs() {
  local dir
  local -a dirs=()
  IFS=: read -r -a dirs <<<"${DOTFILES_LOCAL_INPUTS:-}"
  for dir in ${dirs[@]+"${dirs[@]}"}; do
    git config --system --get-all safe.directory | grep -qxF -- "$dir" ||
      git config --system --add safe.directory "$dir"
  done
}

# Download caches that survive containers (never config): Nix's fetcher and
# eval caches and Neovim's plugins.
link_caches() {
  install -d -o "$USER_NAME" -g "$USER_NAME" "$CACHE/nix" "$CACHE/nvim-data" "$CACHE/nvim-cache" \
    "$USER_HOME/.cache" "$USER_HOME/.local/share"
  local pair
  for pair in ".cache/nix:nix" ".local/share/nvim:nvim-data" ".cache/nvim:nvim-cache"; do
    [[ -e "$USER_HOME/${pair%%:*}" ]] || runuser -u "$USER_NAME" -- ln -s "$CACHE/${pair#*:}" "$USER_HOME/${pair%%:*}"
  done
}

# copy_repo: /src -> ~/Projects/dotfiles, fresh git index so the flake sees
# untracked files too. docs/screenshots and .git stay behind (60 MB each;
# nothing reads them, and the flake would copy them into the store on every
# changed run), and so do the build outputs (target, install/out, iso) and
# the agents' worktrees in .claude: gigabytes, still being written while the
# copy reads them.
copy_repo() {
  local t0
  t0="$(now)"
  [[ -f $SRC/flake.nix ]] || die "/src has no flake.nix (DOTFILES_SRC wrong?)"
  rm -rf "$REPO"
  install -d -o "$USER_NAME" -g "$USER_NAME" "$REPO"
  tar -C "$SRC" --exclude=./.git --exclude=./docs/screenshots --exclude='._*' --exclude=.DS_Store \
    --exclude=./test/out --exclude=./target --exclude=./install/out --exclude=./iso --exclude=./.claude \
    -cf - . | runuser -u "$USER_NAME" -- tar -C "$REPO" -xf -
  as_user sh -c "cd '$REPO' && git init -q -b main && git add -A && \
    git -c user.name=test -c user.email=test@vm commit -q -m 'test snapshot' --no-gpg-sign"
  note "repo copied in $(since "$t0") s"
}

# switch: the real system/60-nix.sh (home-manager switch for kuba@vm, the
# lock screen's pam_fprintd link, GPU setup for Nix apps, zsh) and the
# lock-screen half of system/70-bitwarden.sh (pam_u2f link). Both are
# idempotent; on a warm volume they only build what changed.
switch() {
  local t0 log="$LOGS/switch.log"
  t0="$(now)"
  say "home-manager switch (system/60-nix.sh --host vm)"
  # From a bash login shell, the way you'd run them on a tty.
  if ! as_user bash -l -c "cd '$REPO' && system/60-nix.sh --host vm" >"$log" 2>&1; then
    tail -40 "$log" >&2
    die "system/60-nix.sh failed (full log: $log)"
  fi
  if ! as_user bash -l -c "cd '$REPO' && system/70-bitwarden.sh --no-desktop --no-cli" >>"$log" 2>&1; then
    tail -30 "$log" >&2
    die "system/70-bitwarden.sh --no-desktop --no-cli failed (full log: $log)"
  fi
  # The GPU setup wrote /etc/tmpfiles.d/non-nixos-gpu.conf; apply it like boot.
  if [[ -f /etc/tmpfiles.d/non-nixos-gpu.conf && ! -e /run/opengl-driver ]]; then
    systemd-tmpfiles --create /etc/tmpfiles.d/non-nixos-gpu.conf >>"$log" 2>&1 || true
  fi
  SWITCH_SECONDS="$(since "$t0")"
  note "switched in $SWITCH_SECONDS s (log: $log)"
}

# prepare: everything a test run needs before its checks.
prepare() {
  start_nix_daemon
  boot_like
  trust_local_inputs
  link_caches
  copy_repo
  switch
}

# nix_root OUTLINK INSTALLABLE...: realise into the volume with a GC root.
nix_root() {
  local name="$1"
  shift
  install -d "$GCROOTS"
  "$NIX_BIN/nix" build --out-link "$GCROOTS/$name" "$@"
}
