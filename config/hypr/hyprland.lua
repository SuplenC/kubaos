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
require("accents")
require("looks")
require("rules")
require("keybinds")
require("autostart")

-- The user layer, last so it wins (docs/CONTRACTS.md, "Defaults and the user
-- layer"). Not a require: the file isn't on package.path, and an error in it
-- must not take the rest down. dot-config creates it on install.
local user_file = require("user_layer").file("hypr", "hyprland.lua")
local f = io.open(user_file, "r")
if f then
  f:close()
  local ok, err = pcall(dofile, user_file)
  if not ok then
    local msg = tostring(err):gsub("'", "'\\''")
    hl.exec_cmd("notify-send -u critical 'You broke your Hyprland config' '" .. msg .. "'")
  end
end

-- The power stop's refresh rate and VRR on the laptop panel, over the panel's
-- rule from the user layer too (monitors.lua), so a reload keeps the stop.
pcall(dot.power_panel)
