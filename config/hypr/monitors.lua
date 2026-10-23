-- Monitors. https://wiki.hypr.land/Configuring/Monitors/
--
-- Every screen starts at its preferred mode, placed and scaled by Hyprland.
-- A real layout (which screen sits where, its refresh rate and scale) is the
-- user's, in ~/.config/kubaos/hypr/hyprland.lua; README.md shows how.

dot.monitors = {
  laptop = "eDP-1",
}

local m = dot.monitors

-- The lid script (dot-hypr-lid) creates this flag when the lid is closed and
-- an external is connected; a reload then keeps eDP-1 off. See README.md.
local runtime = os.getenv("XDG_RUNTIME_DIR") or "/tmp"
local lid_flag = io.open(runtime .. "/dot-hypr/laptop-panel-off", "r")
local laptop_off = lid_flag ~= nil
if lid_flag then lid_flag:close() end

if laptop_off then
  hl.monitor({ output = m.laptop, disabled = true })
end

hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })
