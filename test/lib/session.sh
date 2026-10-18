# shellcheck shell=bash disable=SC2012
# A headless Wayland session as kuba: sway (pixman renderer, Docker has no
# DRM device) or Hyprland (when the image's hyprland stage provides it), a
# session D-Bus, and Quickshell from the home-manager generation rendering on
# Mesa llvmpipe. Sourced after harness.sh.

SESSION_ENV=$RUNTIME_DIR/dotfiles-test-session.env
QS_LOG=$LOGS/quickshell.log
HYPR_SESSION=/usr/local/lib/dotfiles-test/hyprland-session

# session_start [sway|hyprland] [WIDTHxHEIGHT] [SCALE]
session_start() {
  local compositor="${1:-sway}" size="${2:-1920x1080}" scale="${3:-1}"
  rm -f "$SESSION_ENV"
  # Audio for the OSD and the volume widgets (systemd --user starts these on
  # the laptop). Best effort: the shell copes without them.
  if command -v pipewire >/dev/null && ! pgrep -u "$USER_NAME" -x pipewire >/dev/null; then
    as_user pipewire >"$LOGS/pipewire.log" 2>&1 &
    sleep 0.5
    as_user wireplumber >>"$LOGS/pipewire.log" 2>&1 &
  fi
  case "$compositor" in
    sway)
      sed -e "s/@SIZE@/$size/" -e "s/@SCALE@/$scale/" "$SRC/test/lib/sway.conf" >"$LOGS/sway.conf"
      chmod 0644 "$LOGS/sway.conf"
      as_user env WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 \
        sway -c "$LOGS/sway.conf" >"$LOGS/sway.log" 2>&1 &
      for _ in $(seq 100); do
        [[ -S $RUNTIME_DIR/wayland-1 ]] && break
        sleep 0.1
      done
      [[ -S $RUNTIME_DIR/wayland-1 ]] || { tail -20 "$LOGS/sway.log" >&2; die "sway didn't start"; }
      local swaysock
      swaysock="$(ls "$RUNTIME_DIR"/sway-ipc.*.sock 2>/dev/null | head -1)"
      printf 'WAYLAND_DISPLAY=wayland-1\nSWAYSOCK=%s\n' "$swaysock" >"$SESSION_ENV"
      ;;
    hyprland)
      [[ -x $HYPR_SESSION ]] ||
        die "no Hyprland in this image yet: the hyprland stage in test/Dockerfile is a TODO (docs/TESTING.md)"
      as_user "$HYPR_SESSION" "$SESSION_ENV" >"$LOGS/hyprland.log" 2>&1 &
      for _ in $(seq 300); do
        [[ -s $SESSION_ENV ]] && break
        sleep 0.1
      done
      [[ -s $SESSION_ENV ]] || { tail -20 "$LOGS/hyprland.log" >&2; die "Hyprland didn't start"; }
      ;;
    *) die "unknown compositor $compositor" ;;
  esac
  printf 'XDG_SESSION_TYPE=wayland\nQT_QPA_PLATFORM=wayland\n' >>"$SESSION_ENV"
  chown "$USER_NAME:" "$SESSION_ENV"
}

# in_session CMD: CMD in kuba's login zsh with the session's environment.
in_session() {
  zrun "set -a; . $SESSION_ENV; set +a; $1"
}

# qs_start: Quickshell with the config from the generation, log in $QS_LOG. Waits
# until the config has loaded (or failed).
qs_start() {

  : >"$QS_LOG"
  chown "$USER_NAME:" "$QS_LOG"
  in_session "exec qs --no-color >$QS_LOG 2>&1" &
  for _ in $(seq 600); do
    grep -q -E 'Configuration Loaded|Failed to load configuration' "$QS_LOG" && break
    sleep 0.1
  done
  grep -q 'Configuration Loaded' "$QS_LOG"
}

# qs_problems: QML errors in the log (empty when clean): anything pointing
# at a .qml file or a JS error, and config load failures. Missing system
# services (NetworkManager, BlueZ, UPower, portals, Hyprland under sway) only
# log warnings; qs_env_warnings lists those.
qs_problems() {
  grep -v -E '^ *INFO' "$QS_LOG" | grep -E '\.qml(:[0-9]+)?|qrc:|ReferenceError|TypeError|SyntaxError|is not defined|is not a type|Cannot assign|Unable to assign|Failed to load configuration|module ".*" is not installed' || true
}
qs_env_warnings() {
  grep -E 'WARN|ERROR' "$QS_LOG" | grep -v -E '\.qml(:[0-9]+)?|qrc:' || true
}

session_stop() {
  pkill -u "$USER_NAME" -x qs 2>/dev/null || true
  pkill -u "$USER_NAME" -x quickshell 2>/dev/null || true
  pkill -u "$USER_NAME" -x ukiyo 2>/dev/null || true
  pkill -u "$USER_NAME" -x sway 2>/dev/null || true
  pkill -u "$USER_NAME" -x wireplumber 2>/dev/null || true
  pkill -u "$USER_NAME" -x pipewire 2>/dev/null || true
  pkill -u "$USER_NAME" -f "$HYPR_SESSION" 2>/dev/null || true
  rm -f "$SESSION_ENV"
}
