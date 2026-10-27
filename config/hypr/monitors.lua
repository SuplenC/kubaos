-- Monitors. https://wiki.hypr.land/Configuring/Monitors/
--
-- Every screen starts at its preferred mode, placed and scaled by Hyprland.
-- A real layout (which screen sits where, its refresh rate and scale) is the
-- user's, in ~/.config/kubaos/desktop/hypr/hyprland.lua; README.md shows how.

dot.monitors = {
  laptop = "eDP-1",
}

local m = dot.monitors

-- The power stops (crates/kubaos-power) set the laptop panel's refresh rate
-- and VRR. `hl.monitor` is wrapped to note each output's rule as the config
-- gives it, the user's included, so the stop changes only the rate and VRR of
-- the laptop's own rule and keeps its position, scale and everything else.
-- Another screen's rule is never touched.
local power = dot.power or {}
dot.power = power
power.rules = {}
if hl.monitor ~= power.recording then
  power.original = hl.monitor
  power.recording = function(spec)
    if type(spec) == "table" and spec.output then
      power.rules[spec.output] = spec
    end
    return power.original(spec)
  end
  hl.monitor = power.recording
end

local runtime = os.getenv("XDG_RUNTIME_DIR") or "/tmp"

-- What the power stop wants of the panel, from the file its service writes
-- (crates/kubaos-power/src/panel.rs): `refresh=60` or `refresh=native`, and
-- `vrr=0` or `vrr=1`. Nil without the file: no service, no change.
local function power_wanted()
  local file = io.open(runtime .. "/kubaos-power/panel", "r")
  if not file then
    return nil
  end
  local text = file:read("a")
  file:close()
  local refresh = text:match("refresh=(%w+)")
  local vrr = tonumber(text:match("vrr=(%d)"))
  if not refresh or not vrr then
    return nil
  end
  return { refresh = refresh, vrr = vrr }
end

-- The laptop's rule with the power stop's rate and VRR: the user's rule for
-- the panel, else the rule every screen falls back to. Nil when there is
-- nothing to change: no stop, a panel the lid turned off, or no size to give
-- a rate to.
function dot.power_rule()
  local wanted = power_wanted()
  if not wanted then
    return nil
  end
  local base = power.rules[m.laptop] or power.rules[""] or { mode = "preferred", position = "auto", scale = "auto" }
  if base.disabled then
    return nil
  end
  local rule = {}
  for key, value in pairs(base) do
    rule[key] = value
  end
  rule.output = m.laptop
  rule.vrr = wanted.vrr
  if wanted.refresh ~= "native" then
    -- A rate needs the mode's size: the rule's own, else the panel's as it runs.
    local size = type(base.mode) == "string" and base.mode:match("^(%d+x%d+)")
    if not size then
      local panel = hl.get_monitor(m.laptop)
      if not panel then
        return nil
      end
      size = panel.width .. "x" .. panel.height
    end
    rule.mode = size .. "@" .. wanted.refresh
  end
  return rule
end

-- Applies the power stop's rate and VRR to the laptop panel. The power
-- service runs it through `hyprctl eval 'dot.power_panel()'` at every stop,
-- and hyprland.lua at every config load, after the user's rules. A rule that
-- changes only VRR is no new mode, so Hyprland turns VRR on or off without a
-- modeset; a new rate is seamless where amdgpu's FreeSync video modes are on
-- (system/files/modprobe.d/kubaos-amdgpu.conf), which the service checks
-- before it asks for one.
function dot.power_panel()
  local rule = dot.power_rule()
  if rule then
    power.original(rule)
  end
end

-- The lid script (dot-hypr-lid) creates this flag when the lid is closed and
-- an external is connected; a reload then keeps eDP-1 off. See README.md.
local lid_flag = io.open(runtime .. "/dot-hypr/laptop-panel-off", "r")
local laptop_off = lid_flag ~= nil
if lid_flag then lid_flag:close() end

if laptop_off then
  hl.monitor({ output = m.laptop, disabled = true })
end

hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })
