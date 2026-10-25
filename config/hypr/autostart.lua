-- Autostart. Runs once per Hyprland start (not on config reload).

-- Polkit agent. Exactly one agent may register per session, and this toggle
-- must agree with Quickshell's `Settings.polkitAgent`
-- (config/quickshell/settings/Settings.qml):
--   "hyprpolkitagent" (default) start the apt agent. Use while
--                     Settings.polkitAgent is false (its default).
--   "quickshell"      never start hyprpolkitagent; the shell's agent is used.
--                     Switch both together: set Settings.polkitAgent to true
--                     AND this to "quickshell", then restart the session.
--   "auto"            start hyprpolkitagent only if Quickshell is not running
--                     5 s after login. Only makes sense with
--                     Settings.polkitAgent = true.
local POLKIT_AGENT = "hyprpolkitagent"

local function start(cmd)
  hl.exec_cmd(cmd)
end

hl.on("hyprland.start", function()
  -- Hand the session environment (WAYLAND_DISPLAY, HYPRLAND_INSTANCE_SIGNATURE,
  -- the vars from env.lua) to systemd --user and D-Bus activated services,
  -- so portals, keyring and apps started via D-Bus see the right display.
  -- Every exec runs in parallel, so the polkit agent (a systemd --user unit
  -- that needs WAYLAND_DISPLAY) is started in the same command, after this.
  local session_env = "systemctl --user import-environment WAYLAND_DISPLAY XDG_CURRENT_DESKTOP XDG_SESSION_TYPE XDG_SESSION_DESKTOP "
    .. "HYPRLAND_INSTANCE_SIGNATURE PATH QT_QPA_PLATFORM QT_QPA_PLATFORMTHEME GDK_BACKEND "
    .. "ELECTRON_OZONE_PLATFORM_HINT XCURSOR_SIZE XCURSOR_THEME HYPRCURSOR_SIZE HYPRCURSOR_THEME; "
    .. "dbus-update-activation-environment --systemd --all; "
    -- Then graphical-session.target (through hyprland-session.target,
    -- home/desktop.nix): xdg-desktop-portal won't start without it.
    .. "systemctl --user start hyprland-session.target"
  if POLKIT_AGENT == "hyprpolkitagent" then
    start(session_env .. "; systemctl --user start hyprpolkitagent")
  else
    start(session_env)
  end

  -- keyd starts with the machine for the Mac-style accents on Left Alt;
  -- this stops it again when the user turned them off
  -- (~/.config/kubaos/desktop/keyboard/keyboard.toml).
  start("kubaos keyboard accents apply")

  -- Secrets (Chrome passwords, git credentials via libsecret).
  start("gnome-keyring-daemon --start --components=secrets")

  -- Bitwarden desktop: its SSH agent (~/.bitwarden-ssh-agent.sock, what
  -- SSH_AUTH_SOCK points at; home/secrets.nix) only answers while the app
  -- runs. Hyprland ignores XDG autostart, so the app's own "start on login"
  -- does nothing here. Turn on "Start to tray icon" in its settings to start
  -- hidden. Not installed (arm64 VM): the exec fails quietly.
  -- While KubaOS first-boot setup is pending (its user unit is installed and
  -- its marker isn't there yet), setup has the screen to itself and starts
  -- Bitwarden when it closes (crates/kubaos-kobo/src/setup).
  start('[ -e /usr/lib/systemd/user/kubaos-setup.service ] && '
    .. '[ ! -e "${XDG_STATE_HOME:-$HOME/.local/state}/kubaos/setup-done" ] || exec bitwarden')

  -- Regenerate colors from the current wallpaper (or copy the default
  -- palette on first boot); a few ms no-op when nothing changed since the
  -- last run. Runs in parallel with the shell: Quickshell watches
  -- colors.json and repaints when it lands.
  start("dot-theme apply")

  -- Desktop shell (bar, Command menu, notifications, OSD, lock, wallpaper).
  start("qs")

  -- Idle: lock 5 min, screens off 10 min, suspend 30 min on battery.
  start("hypridle")

  -- Runs `dot-hypr-lid sync` at login and on every monitor hotplug, so the
  -- laptop panel comes back when the last external goes with the lid closed.
  start("dot-hypr-workspaces")

  if POLKIT_AGENT == "auto" then
    -- pgrep -x matches the process name; the Nix wrapper shows up as
    -- ".quickshell-wra" (names are cut at 15 chars).
    start("sleep 5; pgrep -x qs >/dev/null || pgrep -x quickshell >/dev/null"
      .. " || pgrep -x .quickshell-wra >/dev/null || systemctl --user start hyprpolkitagent")
  end
end)

-- Every way out KubaOS offers (`dot-session leave`, the fallback exit bind in
-- fallback.lua) has already stopped graphical-session.target, while Hyprland
-- still answered the portals and the polkit agent. This catches the rest (a
-- SIGTERM, a crash), so the next session starts the target afresh.
-- HAZARD: keep --no-block. Hyprland's main loop stops here until the command
-- returns, and a stop that waits for its Wayland clients would wait on it.
hl.on("hyprland.shutdown", function()
  os.execute("systemctl --user --no-block stop graphical-session.target")
end)
