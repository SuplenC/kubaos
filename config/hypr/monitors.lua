-- Monitors. https://wiki.hypr.land/Configuring/Monitors/
--
-- Laptop panel: eDP-1, 2880x1800@120 (14", ~243 ppi).
-- Scale 1.5 -> 1920x1200 logical: the same density as the 1080p externals,
-- so the shell, fonts and terminal sizes are tuned once for "Full HD at scale
-- 1" and look the same on both. 2880/1.5 and 1800/1.5 are whole numbers and
-- 1.5 is 180/120 (the wp-fractional-scale step), so Hyprland takes it as is.
-- Scale 2 (1440x900) made everything look oversized; for a bit more room
-- than 1.5, 1.6 is clean too (1800x1125). Other values Hyprland can't divide
-- evenly (1.75: 1645.7 px) are replaced by the nearest clean scale with a
-- warning. Native Wayland apps (Qt, GTK, Chrome, Ghostty, Quickshell) render
-- at 1.5 directly; XWayland apps are upscaled (see looks.lua).
--
-- Externals: two LG UltraGear 1920x1080@120 on the USB4 dock, matched by
-- description so it does not matter which dock port each one lands on.
--
-- HOW TO FILL IN THE DESCRIPTIONS
--   1. Plug the dock in and run:  hyprctl monitors all
--   2. Each monitor prints a line like
--        description: LG Electronics LG ULTRAGEAR 312NTXX1A234
--      The two UltraGears share make and model, so the serial at the end is
--      what tells them apart. Copy the whole line after "description: ".
--   3. Paste them below as desc:<description> (external_1 = the one left of
--      external_2), and fix the positions so they match the physical layout
--      on your desk. Workspaces don't depend on this: workspaces.lua knows
--      each screen by the same description.
--   4. Save. Hyprland reloads by itself (or run: hyprctl reload).
-- Until then the placeholders match nothing and the catch-all rule at the
-- bottom places any monitor automatically.

dot.monitors = {
  laptop = "eDP-1",
  external_1 = "desc:LG Electronics LG ULTRAGEAR REPLACE-WITH-SERIAL-1",
  external_2 = "desc:LG Electronics LG ULTRAGEAR REPLACE-WITH-SERIAL-2",
}

local m = dot.monitors

-- Layout (logical pixels): laptop on the left, the two externals side by
-- side to its right, all bottom-aligned (the laptop is 120 px taller).
--
--   +-----------+
--   |           +-----------+-----------+
--   |   eDP-1   | external_1| external_2|
--   | 1920x1200 | 1920x1080 | 1920x1080 |
--   +-----------+-----------+-----------+
--   0           1920        3840        5760

-- The lid script (dot-hypr-lid) creates this flag when the lid is closed and
-- an external is connected; a reload then keeps eDP-1 off. See README.md.
local runtime = os.getenv("XDG_RUNTIME_DIR") or "/tmp"
local lid_flag = io.open(runtime .. "/dot-hypr/laptop-panel-off", "r")
local laptop_off = lid_flag ~= nil
if lid_flag then lid_flag:close() end

if laptop_off then
  hl.monitor({ output = m.laptop, disabled = true })
else
  hl.monitor({ output = m.laptop, mode = "2880x1800@120", position = "0x0", scale = 1.5 })
end

hl.monitor({ output = m.external_1, mode = "1920x1080@120", position = "1920x120", scale = 1 })
hl.monitor({ output = m.external_2, mode = "1920x1080@120", position = "3840x120", scale = 1 })

-- Anything else (projectors, a TV, a monitor at a client's office).
hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })
