-- The active window's neon focus, the components' focus tube on its border:
-- the border in `primary` and a glow outside it, drawn as the window's shadow
-- in `primary`. Inactive windows keep a neutral border and shadow.
--
-- Focus moving to another window strikes the tube on, as a control's does
-- (`qml/KubaOS/Components/FocusRing.qml`): the border and the glow step
-- through `neon.strike` over `duration.medium`, once, then stay lit. The
-- window's content never dims. The user layer's `focusStutter`
-- (desktop/quickshell/settings.json, the Command menu's "Focus stutter" Item)
-- turns it off; then Hyprland's own border and shadow fades light the window.
--
-- Hyprland's curves only ease, so the steps are a repeating `hl.timer` that
-- sets the active colours, armed only while a tube strikes. Each step is
-- instant because the `border` and `fadeShadow` fades are off for the strike:
-- a fade would smear the steps into one ramp. They come back as looks.lua set
-- them (`fade`) when the strike ends, early or not; a config reload drops the
-- timer and sets every colour and fade again.
--
-- https://wiki.hypr.land/Configuring/Basics/Variables/#decoration
-- https://wiki.hypr.land/Configuring/Basics/Animations/

local tokens = require("tokens")
local colors = require("colors")
local user_layer = require("user_layer")

local M = {}

-- Hyprland's focus reasons that don't strike, as `window.active` passes them
-- (`eFocusReason`, src/desktop/state/FocusState.hpp): a workspace switch and a
-- special workspace show a window that was already focused there, and a new
-- window arrives lit, as a control that appears focused does.
local QUIET = {
  [11] = true,
  [12] = true,
  [16] = true,
}

local strike = tokens.neon.strike
local step_ms = math.max(1, tokens.duration.medium // #strike)

-- The glow's opacity `d` logical pixels outside the tube: `kubaos-ui`'s
-- `focus::glow`, a near and a far glow each fading as the square of the way
-- out.
local function glow(d)
  local focus = tokens.focus
  local function fade(reach, opacity)
    return d >= reach and 0 or opacity * (1 - d / reach) ^ 2
  end
  return 1 - (1 - fade(focus.near, focus.near_opacity)) * (1 - fade(focus.far, focus.far_opacity))
end

-- Hyprland's shadow falls off from its opacity at the window's edge as
-- `(1 - d / range) ^ render_power`, one curve for every window. It reaches
-- as far as the far glow, at the power (1 to 4, Hyprland's limits) that
-- comes closest to the two glows over that reach.
local function fit()
  local range = tokens.focus.far
  local edge = glow(0)
  local best, power = math.huge, 1
  for p = 1, 4 do
    local err = 0
    for i = 0, range * 4 do
      local d = i / 4
      err = err + (edge * (1 - d / range) ^ p - glow(d)) ^ 2
    end
    if err < best then
      best, power = err, p
    end
  end
  return { range = range, power = power, alpha = edge }
end

M.shadow = fit()

local function channels(hex)
  local r, g, b = hex:match("^#?(%x%x)(%x%x)(%x%x)$")
  return tonumber(r, 16) / 255, tonumber(g, 16) / 255, tonumber(b, 16) / 255
end

-- `to` over `from` at `level`, both "#rrggbb" at their own opacity, mixed
-- premultiplied as Hyprland fades one colour into another: the tube at a
-- level is drawn over the inactive look, not over nothing.
local function mix(from, from_alpha, to, to_alpha, level)
  local fr, fg, fb = channels(from)
  local tr, tg, tb = channels(to)
  local a = from_alpha * (1 - level) + to_alpha * level
  if a <= 0 then
    return "rgba(00000000)"
  end
  local function channel(f, t)
    return math.floor((f * from_alpha * (1 - level) + t * to_alpha * level) / a * 255 + 0.5)
  end
  return string.format("rgba(%02x%02x%02x%02x)", channel(fr, tr), channel(fg, tg), channel(fb, tb),
    math.floor(a * 255 + 0.5))
end

-- An inactive window's border, the highest container at 80%, and its faint
-- neutral shadow.
local INACTIVE_BORDER = 0.8
local INACTIVE_SHADOW = 1 / 3

function M.inactive(c)
  return {
    border = colors.rgba(c.surface_container_highest, string.format("%02x", math.floor(INACTIVE_BORDER * 255 + 0.5))),
    shadow = colors.rgba(c.shadow, string.format("%02x", math.floor(INACTIVE_SHADOW * 255 + 0.5))),
  }
end

-- The active window's border and shadow with the tube at `level`, 0 (an
-- inactive window's) to 1 (lit).
function M.colors(c, level)
  return {
    border = mix(c.surface_container_highest, INACTIVE_BORDER, c.primary, 1, level),
    shadow = mix(c.shadow, INACTIVE_SHADOW, c.primary, M.shadow.alpha, level),
  }
end

-- The palette in use, the fades as looks.lua sets them, and the strike's
-- state: the step it is on (nil while lit) and its timer, made at the first
-- strike.
local palette = colors.defaults
local fades = {}
local step = nil
local timer = nil

local function paint(level)
  local active = M.colors(palette, level)
  hl.config({
    general = { col = { active_border = active.border } },
    group = { col = { border_active = active.border } },
    decoration = { shadow = { color = active.shadow } },
  })
end

local function set_fades(on)
  for _, spec in ipairs(fades) do
    hl.animation(on and spec or { leaf = spec.leaf, enabled = false })
  end
end

-- Ends a strike lit, its timer disarmed and the fades back. Each part runs
-- even when another fails, so no window is left dim.
local function finish()
  if not step then
    return
  end
  step = nil
  pcall(function()
    timer:set_enabled(false)
  end)
  pcall(paint, 1)
  pcall(set_fades, true)
end

local function tick()
  local ok = pcall(function()
    step = step + 1
    if step > #strike then
      finish()
    else
      paint(strike[step])
    end
  end)
  if not ok then
    finish()
  end
end

local function start()
  local ok = pcall(function()
    if not step then
      set_fades(false)
    end
    step = 1
    paint(strike[step])
    if timer then
      timer:set_enabled(true)
    else
      timer = hl.timer(tick, { timeout = step_ms, type = "repeat" })
    end
  end)
  if not ok then
    finish()
  end
end

-- The stutter as the user layer's settings.json sets it: on unless a line
-- that isn't a `//` comment says `"focusStutter": false`, as
-- `config/quickshell/settings/Settings.qml` reads it. Read at each focus
-- change, so the Command menu's Item takes effect at the next one.
-- TODO(#154): reduce motion turns the strike off too; the shell's setting doesn't exist yet.
local function stutters()
  local f = io.open(user_layer.file("quickshell", "settings.json"), "r")
  if not f then
    return true
  end
  local on = true
  for line in f:lines() do
    if not line:match("^%s*//") then
      local value = line:match('"focusStutter"%s*:%s*(%a+)')
      if value then
        on = value ~= "false"
      end
    end
  end
  f:close()
  return on
end

-- The workspaces on screen as of the last workspace event. A workspace
-- switch focuses its window before it says the workspace is active, and the
-- pointer can land on another window of it in between: that window's
-- workspace isn't here yet, so it doesn't strike.
local shown = {}

local function read_shown()
  shown = {}
  for _, monitor in ipairs(hl.get_monitors() or {}) do
    for _, workspace in ipairs({ monitor.active_workspace, monitor.active_special_workspace }) do
      if workspace then
        shown[workspace.id] = true
      end
    end
  end
end

-- The window that had focus last. Focus leaving every window (a lock, a
-- menu) keeps it, so focus coming back to it doesn't strike.
local last = hl.get_active_window()
last = last and last.address

local function on_active(window, reason)
  if not window then
    return
  end
  local moved = window.address ~= last
  last = window.address
  if not moved then
    return
  end
  local workspace = window.workspace
  if QUIET[reason] or not (workspace and shown[workspace.id]) or not stutters() then
    finish()
    return
  end
  start()
end

-- An animation looks.lua sets that a strike turns off and back on: the
-- border's and the shadow's fades.
function M.fade(spec)
  fades[#fades + 1] = spec
  hl.animation(spec)
end

-- Paints the active window's colours from palette `c`, at the strike's step
-- when one is running.
function M.use(c)
  palette = c
  paint(step and strike[step] or 1)
end

read_shown()
hl.on("window.active", on_active)
for _, event in ipairs({ "workspace.active", "workspace.special_active", "monitor.added", "monitor.removed" }) do
  hl.on(event, read_shown)
end

return M
