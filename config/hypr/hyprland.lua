-- Hyprland 0.56+ Lua config. Entry point.
--
-- Each require() runs in its own protected scope (Hyprland wraps require), so
-- an error in one module pops a notification but does not stop the others.
-- fallback.lua loads first on purpose: its binds exist even if every other
-- module is broken.
--
-- API reference: /usr/share/hypr/stubs/ (LuaLS stubs shipped with Hyprland),
-- https://wiki.hypr.land/Configuring/ (pick your version in the selector).

-- Shared namespace for helpers that other modules (and `hyprctl eval`) call.
dot = dot or {}

require("fallback")
require("env")
require("monitors")
require("workspaces")
require("input")
require("looks")
require("rules")
require("keybinds")
require("autostart")

-- Optional machine-local overrides, git-ignored (see README.md).
local config_home = os.getenv("XDG_CONFIG_HOME") or ((os.getenv("HOME") or "") .. "/.config")
local local_file = io.open(config_home .. "/hypr/local.lua", "r")
if local_file then
  local_file:close()
  require("local")
end
