# shellcheck shell=bash
# The user hooks of the Apps catalog's setups (docs/CONTRACTS.md, "The Apps catalog"): for a
# setup's `user_hooks = "<hooks>"`, install_<hooks> and remove_<hooks>, which apps/user-helper runs
# as the user. install_<hooks> gets the setup's checked download and its version when the setup
# has one. Each is safe to run again, and leaves the setup's marker there exactly while its parts
# are.

# T3 Code as a service (apps/t3-code.toml): T3's `t3` runtime under the T3 home, its background
# service as a systemd user unit on this machine only, and the desktop app as its client. Every
# step is T3's own, as its installer and `t3 up` take them, so `t3 update` and `t3 uninstall` know
# what is there.
# https://github.com/pingdotgg/t3code/blob/main/docs/user/background-service.md

# t3_home: the T3 home, as T3 reads it.
t3_home() {
  printf '%s' "${T3CODE_HOME:-$HOME/.t3}"
}

# install_t3_service ARCHIVE VERSION
install_t3_service() {
  local archive="$1" version="$2" home target staging
  home="$(t3_home)"
  command -v jq >/dev/null || { echo "T3 Code as a service needs jq, which KubaOS's home has (home/cli.nix)" >&2; return 1; }
  # HAZARD: the desktop app's own server holds the T3 home while it is open, and a service started
  # meanwhile restarts and fails until it quits.
  if pgrep -u "$(id -u)" -x t3code >/dev/null; then
    echo "T3 Code is open: quit it, then install T3 Code as a service again" >&2
    return 1
  fi
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
  "$target/t3" install
  # A new service answers on this machine only (T3CODE_LAN=false in service.env) and has no T3
  # Connect: `t3 up`, which also turns T3 Connect on for a new machine, is the user's to run.
  "$target/t3" service install
  t3_client_to_service "$home" "$target/t3"
}

# t3_client_to_service HOME T3: the desktop app without a server of its own, and a one-time
# pairing link to the service opened in it, which T3 Code asks the user to confirm once. T3 keeps
# its saved connections encrypted, so nothing else can add one.
t3_client_to_service() {
  local home="$1" t3="$2" origin url link
  origin="$(t3_service_origin "$home")" || {
    echo "T3 Code's service didn't start: \`t3 service status\` says why" >&2
    return 1
  }
  t3_local_server "$home/userdata/desktop-settings.json" off
  url="$("$t3" auth pairing create --json --ttl 15m --label "T3 Code on this machine" --base-url "$origin" |
    jq -r '.pairUrl // empty')"
  [[ -n $url ]] || { echo "t3 gave no pairing link for its service" >&2; return 1; }
  link="t3code://pair?pairingUrl=$(jq -rn --arg url "$url" '$url | @uri')"
  if [[ -n ${WAYLAND_DISPLAY:-} ]] && command -v t3code >/dev/null; then
    setsid -f t3code "$link" >/dev/null 2>&1 </dev/null
    echo "T3 Code opens and asks to add the backend on this machine: say yes, and it connects to the service from then on"
  else
    echo "Open T3 Code and run \`t3 pair\` to connect it to the service on this machine"
  fi
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
# their other keys kept. On is T3's default, which the file then leaves out.
t3_local_server() {
  local file="$1" filter tmp
  case "$2" in
    on) [[ -f $file ]] || return 0; filter='del(.localEnvironmentEnabled)' ;;
    off) filter='.localEnvironmentEnabled = false' ;;
  esac
  mkdir -p "$(dirname "$file")"
  tmp="$(mktemp "$file.XXXXXX")"
  if ! { if [[ -s $file ]]; then jq "$filter" "$file"; else jq -n "{} | $filter"; fi; } >"$tmp"; then
    rm -f "$tmp"
    echo "$file isn't JSON T3 Code wrote; leaving it" >&2
    return 1
  fi
  mv "$tmp" "$file"
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
  elif [[ -e $unit ]]; then
    systemctl --user disable --now t3code.service || true
    rm -f "$unit"
    systemctl --user daemon-reload
  fi
  [[ ! -e $unit ]] || { echo "T3 Code's service is still installed: $unit" >&2; return 1; }
  t3_local_server "$home/userdata/desktop-settings.json" on
}
