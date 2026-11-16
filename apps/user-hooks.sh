# shellcheck shell=bash
# The user hooks of the Apps catalog's setups (docs/CONTRACTS.md, "The Apps catalog"): for a
# setup's `user_hooks = "<hooks>"`, install_<hooks> and remove_<hooks>, which apps/user-helper runs
# as the user, and ready_<hooks> VERB when the setup can't always be installed or taken away, which
# says why not before anything changes. install_<hooks> gets the setup's checked download and its
# version when the setup has one, opens no window unless `setup_window` is 1, and tells the user a
# step left to them with `next`. Each is safe to run again, and an install that fails leaves no
# marker: the setup is installed exactly while its marker is there.

# T3 Code as a service (apps/t3-code.toml): T3's `t3` runtime under the T3 home, its background
# service as a systemd user unit on this machine only, and the desktop app as its client. Every
# step is T3's own, as its installer and `t3 up` take them, so `t3 update` and `t3 uninstall` know
# what is there.
# https://github.com/pingdotgg/t3code/blob/main/docs/user/background-service.md

# t3_home: the T3 home, as T3 reads it.
t3_home() {
  printf '%s' "${T3CODE_HOME:-$HOME/.t3}"
}

# ready_t3_service: T3 Code's desktop app is closed, for an install and a removal alike. HAZARD: its own server holds the T3
# home while it is open: a service started meanwhile restarts and fails until it quits, and the
# app writes its settings back over a removal's when it quits.
ready_t3_service() {
  if pgrep -u "$(id -u)" -x t3code >/dev/null; then
    echo "T3 Code is open: quit it, then try again" >&2
    return 1
  fi
}

# install_t3_service ARCHIVE VERSION
install_t3_service() {
  local archive="$1" version="$2" home target staging
  home="$(t3_home)"
  command -v jq >/dev/null || { echo "T3 Code as a service needs jq, which KubaOS's home has (home/cli.nix)" >&2; return 1; }
  target="$home/runtime/versions/$version"
  # Unpacked the way T3's installer unpacks it (scripts/install.sh upstream), so the service and
  # `t3 update` take this version as their own.
  if [[ "$(cat "$target/.install-complete" 2>/dev/null)" != "$version" ]]; then
    mkdir -p "$home/runtime/versions"
    staging="$(mktemp -d "$home/runtime/versions/.staging-XXXXXX")"
    if ! tar -xzf "$archive" -C "$staging" --strip-components=1 || ! "$staging/t3" --version >/dev/null; then
      rm -rf "$staging"
      echo "the t3 runtime $version doesn't unpack or doesn't run" >&2
      return 1
    fi
    printf '%s\n' "$version" >"$staging/.install-complete"
    rm -rf "$target"
    mv "$staging" "$target"
  fi
  "$target/t3" install || return 1
  t3_linger_note
  # A new service answers on this machine only (T3CODE_LAN=false in service.env) and has no T3
  # Connect: `t3 up`, which also turns T3 Connect on for a new machine, is the user's to run.
  # The unit is the setup's marker, so a later step that fails takes the service away again and
  # the setup isn't left looking installed.
  if ! "$target/t3" service install || ! t3_client_to_service "$home" "$target/t3"; then
    t3_service_away "$home" "$target/t3"
    return 1
  fi
}

# t3_service_away HOME T3: the service stopped and its unit gone, the desktop app's own server
# back, and lingering as it was before KubaOS set the service up. The runtime and the `t3`
# launchers stay, for a retry or `t3 uninstall`.
t3_service_away() {
  local home="$1" t3="$2"
  "$t3" service uninstall || true
  t3_unit_away
  t3_local_server "$home/userdata/desktop-settings.json" on
  t3_linger_restore
}

# t3_client_to_service HOME T3: the desktop app without a server of its own, and, in a graphical
# session that may open a window, a one-time pairing link to the service opened in it, which T3
# Code asks the user to confirm once. T3 keeps its saved connections encrypted, so nothing else can
# add one; without the window, pairing is the user's next step.
t3_client_to_service() {
  local home="$1" t3="$2" origin url link
  origin="$(t3_service_origin "$home")" || {
    echo "T3 Code's service didn't start: \`t3 service status\` says why" >&2
    return 1
  }
  t3_local_server "$home/userdata/desktop-settings.json" off || return 1
  if [[ ${setup_window:-0} != 1 || -z ${WAYLAND_DISPLAY:-} ]] || ! command -v t3code >/dev/null; then
    next "Open T3 Code and run \`t3 pair\` to connect it to the service on this machine"
    return 0
  fi
  url="$("$t3" auth pairing create --json --ttl 15m --label "T3 Code on this machine" --base-url "$origin" |
    jq -r '.pairUrl // empty')"
  [[ -n $url ]] || { echo "t3 gave no pairing link for its service" >&2; return 1; }
  link="t3code://pair?pairingUrl=$(jq -rn --arg url "$url" '$url | @uri')"
  setsid -f t3code "$link" >/dev/null 2>&1 </dev/null
  next "T3 Code opens and asks to add the backend on this machine: say yes, and it connects to the service from then on"
}

# t3_service_origin HOME: the URL the running service answers on, which it writes once it is up.
# Its port is 3773 unless that is taken.
t3_service_origin() {
  local runtime="$1/userdata/server-runtime.json" pid origin tries
  for ((tries = 0; tries < 60; tries++)); do
    if read -r pid origin < <(jq -r 'select(.serviceManaged == true) | "\(.pid) \(.origin)"' "$runtime" 2>/dev/null) &&
      kill -0 "$pid" 2>/dev/null; then
      printf '%s' "$origin"
      return 0
    fi
    sleep 0.5
  done
  return 1
}

# t3_local_server FILE on|off: whether the desktop app runs a server of its own, in its settings,
# their other keys kept. On is T3's default, which the file then leaves out. T3 Code reads a file
# that isn't a JSON object as its defaults, so such a file is on already, and off holds that key
# alone.
t3_local_server() {
  local file="$1" filter tmp
  case "$2" in
    on) [[ -f $file ]] || return 0; filter='del(.localEnvironmentEnabled)' ;;
    off) filter='.localEnvironmentEnabled = false' ;;
  esac
  mkdir -p "$(dirname "$file")"
  tmp="$(mktemp "$file.XXXXXX")"
  if jq -e 'type == "object"' "$file" >/dev/null 2>&1; then
    jq "$filter" "$file" >"$tmp"
  elif [[ $2 == on ]]; then
    rm -f "$tmp"
    return 0
  else
    jq -n "{} | $filter" >"$tmp"
  fi || { rm -f "$tmp"; return 1; }
  mv "$tmp" "$file"
}

# t3_linger_mark: the file that says lingering was off before KubaOS set T3 Code's service up,
# which `t3 service install` turns on. KubaOS's state, never the picks.
t3_linger_mark() {
  printf '%s' "${XDG_STATE_HOME:-$HOME/.local/state}/kubaos/apps/t3-code-linger-was-off"
}

# t3_linger_note: whether the user lingers before the service turns it on, written down when it
# is off. A mark already there stays: a service set up before still owes the user that. When
# logind can't say, nothing is written, and a removal leaves lingering as it is.
t3_linger_note() {
  local mark
  mark="$(t3_linger_mark)"
  [[ $(loginctl show-user "$(id -un)" --property=Linger --value 2>/dev/null) == no ]] || return 0
  mkdir -p "$(dirname "$mark")"
  : >"$mark"
}

# t3_linger_restore: lingering off again when it was off before KubaOS set the service up.
t3_linger_restore() {
  local mark
  mark="$(t3_linger_mark)"
  [[ -e $mark ]] || return 0
  loginctl disable-linger "$(id -un)" || { echo "lingering stays on: \`loginctl disable-linger\` failed" >&2; return 0; }
  rm -f "$mark"
}

# t3_unit_away: T3 Code's user unit stopped and gone, for a service whose runtime is gone or
# didn't take it away.
t3_unit_away() {
  local unit="$HOME/.config/systemd/user/t3code.service"
  [[ -e $unit ]] || return 0
  systemctl --user disable --now t3code.service || true
  rm -f "$unit"
  systemctl --user daemon-reload
}

# t3_runtime HOME: the newest t3 downloaded whole under HOME, if any.
t3_runtime() {
  local dir newest
  newest="$(
    for dir in "$1"/runtime/versions/*/; do
      if [[ "$(cat "$dir.install-complete" 2>/dev/null)" == "$(basename "$dir")" && -x "$dir/t3" ]]; then
        basename "$dir"
      fi
    done | sort -V | tail -n 1
  )"
  [[ -z $newest ]] || printf '%s' "$1/runtime/versions/$newest/t3"
}

remove_t3_service() {
  local home t3 unit="$HOME/.config/systemd/user/t3code.service"
  home="$(t3_home)"
  t3="$(t3_runtime "$home")"
  # T3's uninstall takes the service, the `t3` launchers and every downloaded version, and keeps
  # the projects, threads and settings under userdata, and service.env.
  if [[ -n $t3 ]]; then
    "$t3" uninstall --yes
  fi
  t3_unit_away
  [[ ! -e $unit ]] || { echo "T3 Code's service is still installed: $unit" >&2; return 1; }
  t3_local_server "$home/userdata/desktop-settings.json" on
  t3_linger_restore
}
