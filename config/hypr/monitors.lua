-- Monitors. https://wiki.hypr.land/Configuring/Monitors/
--
-- Every screen starts at its preferred mode, placed and scaled by Hyprland.
-- A real layout (which screen sits where, its refresh rate and scale) is the
-- user's, in ~/.config/kubaos/desktop/hypr/hyprland.lua; README.md shows how.

dot.monitors = {
  laptop = "eDP-1",
}

local m = dot.monitors
local runtime = os.getenv("XDG_RUNTIME_DIR") or "/tmp"

-- The power stops (crates/kubaos-power) set the laptop panel's VRR and, in
-- Super Saver, its refresh rate.
--
-- VRR goes through the global `misc:vrr`, never the panel's rule: Hyprland
-- leaves a screen alone when its new rule differs from the one in use only
-- in `vrr` (CMonitorRule::compare doesn't look at it), so a rule's VRR
-- changes nothing until the next mode change. Setting `misc:vrr` refreshes
-- every screen's VRR at once without a modeset (its value's refresh bit runs
-- ensureVRR), and a screen whose rule names its own `vrr` keeps it. So the
-- panel's rule names none, and every other screen's rule gets `vrr = 0`
-- unless the user's names one: the stops never reach an external screen.
--
-- The rate goes through the panel's own rule, which `hl.monitor` merges
-- into by output name, so the user's position, scale and the rest stay.
local power = {
  -- The panel's mode as the config gives it, which the native rate goes back to.
  base_mode = "preferred",
  -- An `hl.monitor` of the power stop's own, not the config's.
  applying = false,
}
dot.power = power

local original_monitor = hl.monitor
hl.monitor = function(spec)
  if type(spec) ~= "table" or not spec.output then
    return original_monitor(spec)
  end
  if spec.output == m.laptop then
    if spec.mode and not power.applying then
      power.base_mode = spec.mode
    end
    return original_monitor(spec)
  end
  if spec.vrr == nil then
    local own = {}
    for key, value in pairs(spec) do
      own[key] = value
    end
    own.vrr = 0
    return original_monitor(own)
  end
  return original_monitor(spec)
end

-- A screen keeps the VRR of the rule it was set up with until its mode
-- changes, so rules written before this file gave every screen its `vrr`
-- hold until Hyprland starts again. The first config load of a Hyprland runs
-- before any screen is set up; it leaves a mark for this instance, and
-- without the mark the stops keep `misc:vrr` off, so an external screen set
-- up under the old rules never turns VRR on.
local signature = os.getenv("HYPRLAND_INSTANCE_SIGNATURE") or "none"
local vrr_mark = runtime .. "/kubaos-power-vrr-" .. signature
local screens_up = (hl.get_monitors and #(hl.get_monitors() or {}) > 0) or false
if not screens_up then
  local mark = io.open(vrr_mark, "w")
  if mark then
    mark:close()
  end
end
local mark = io.open(vrr_mark, "r")
power.vrr_ready = mark ~= nil
if mark then
  mark:close()
end

-- What the power stop wants of the panel, from the file its service writes
-- (crates/kubaos-power/src/panel.rs): `refresh=60` or `refresh=native`, and
-- `vrr=0` or `vrr=1`. Without the file (no service yet), the native rate with
-- VRR, as Balanced has it.
local function power_wanted()
  local file = io.open(runtime .. "/kubaos-power/panel", "r")
  if not file then
    return { refresh = "native", vrr = 1 }
  end
  local text = file:read("a")
  file:close()
  local refresh = text:match("refresh=(%w+)") or "native"
  local vrr = tonumber(text:match("vrr=(%d)")) or 1
  return { refresh = refresh, vrr = vrr }
end

-- The panel's mode for the stop: the config's, or its size at the stop's rate.
local function power_mode(wanted)
  if wanted.refresh == "native" then
    return power.base_mode
  end
  local size = type(power.base_mode) == "string" and power.base_mode:match("^(%d+x%d+)")
  if not size then
    local panel = hl.get_monitor and hl.get_monitor(m.laptop)
    if not panel then
      return power.base_mode
    end
    size = panel.width .. "x" .. panel.height
  end
  return size .. "@" .. wanted.refresh
end

-- Puts the power stop's VRR and rate on the laptop panel. The power service
-- runs it through `hyprctl eval 'dot.power_panel()'` at every stop, and
-- hyprland.lua at every config load, after the user's rules. A rate is asked
-- for only where amdgpu switches it without a modeset (its FreeSync video
-- modes, system/files/modprobe.d/kubaos-amdgpu.conf), which the service
-- checks first.
function dot.power_panel()
  local wanted = power_wanted()
  hl.config({ misc = { vrr = power.vrr_ready and wanted.vrr or 0 } })
  power.applying = true
  local ok, err = pcall(original_monitor, { output = m.laptop, mode = power_mode(wanted) })
  power.applying = false
  assert(ok, err)
end

-- The panel's own rule, so it never falls back to the rule below, whose
-- `vrr = 0` would hold it off.
hl.monitor({ output = m.laptop, mode = "preferred", position = "auto", scale = "auto" })

-- The lid script (dot-hypr-lid) creates this flag when the lid is closed and
-- an external is connected; a reload then keeps eDP-1 off. See README.md.
local lid_flag = io.open(runtime .. "/dot-hypr/laptop-panel-off", "r")
local laptop_off = lid_flag ~= nil
if lid_flag then lid_flag:close() end

if laptop_off then
  hl.monitor({ output = m.laptop, disabled = true })
end

hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })
