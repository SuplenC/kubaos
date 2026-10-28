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
-- a fade would smear the steps into one ramp. With them off, the window that
-- lost focus drops to its inactive look at once too. The fades come back as
-- `fade` set them one step after the strike ends lit, early or not, so the
-- lit colours land without a fade; a config reload drops the timer and sets
-- every colour and fade again.
--
-- This module owns the active window's border and shadow colours and those
-- two fades: a user layer changes the fades through `dot.focus.fade`.
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

-- Hyprland's shadow falls off from its opacity at the border's outer edge as
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

local function byte(value)
  return math.floor(value * 255 + 0.5)
end

local function channels(hex)
  local r, g, b = hex:match("^#?(%x%x)(%x%x)(%x%x)$")
  return tonumber(r, 16) / 255, tonumber(g, 16) / 255, tonumber(b, 16) / 255
end

-- `to` drawn over `from` with `to` at `level`, each "#rrggbb" at its own
-- opacity: at 0 it is `from`, at 1 `to`.
local function over(from, from_alpha, to, to_alpha, level)
  local fr, fg, fb = channels(from)
  local tr, tg, tb = channels(to)
  local a = from_alpha * (1 - level) + to_alpha * level
  if a <= 0 then
    return "rgba(00000000)"
  end
  local function channel(f, t)
    return byte((f * from_alpha * (1 - level) + t * to_alpha * level) / a)
  end
  return string.format("rgba(%02x%02x%02x%02x)", channel(fr, tr), channel(fg, tg), channel(fb, tb), byte(a))
end

local function rgba(hex, opacity)
  return colors.rgba(hex, string.format("%02x", byte(opacity)))
end

-- An inactive window's border and shadow.
function M.inactive(c)
  return {
    border = rgba(c.surface_container_highest, tokens.opacity.window_border),
    shadow = rgba(c.shadow, tokens.opacity.window_shadow),
  }
end

-- The active window's border and shadow with the tube at `level`, 0 (an
-- inactive window's) to 1 (lit).
function M.colors(c, level)
  return {
    border = over(c.surface_container_highest, tokens.opacity.window_border, c.primary, 1, level),
    shadow = over(c.shadow, tokens.opacity.window_shadow, c.primary, M.shadow.alpha, level),
  }
end

-- The palette in use; the fades by leaf, in the order they were set; and the
-- strike: the step it is on (nil while lit), whether the lit colours are
-- waiting a step for the fades, whether the window striking is in a group,
-- and its timer, made at the first strike.
local palette = colors.defaults
local fades, leaves = {}, {}
local step = nil
local settling = false
local grouped = false
local timer = nil

-- Sets the active window's colours with the tube at `level`, and an active
-- group's at `group_level` (left as it is for nil).
-- HAZARD: each `hl.config` of a colour at run time refreshes every window's
-- rules and decorations and draws two full frames on every screen, ten times a
-- strike. The group's colour is set only when a grouped window strikes or the
-- palette changes, as it also redraws every groupbar's gradients.
local function paint(level, group_level)
  local active = M.colors(palette, level)
  local group = group_level and { col = { border_active = M.colors(palette, group_level).border } }
  hl.config({
    general = { col = { active_border = active.border } },
    group = group,
    decoration = { shadow = { color = active.shadow } },
  })
end

-- The group's level at the strike's `level`: nil unless the window striking
-- is in a group.
local function group_at(level)
  return grouped and level or nil
end

local function set_fades(on)
  for _, leaf in ipairs(leaves) do
    hl.animation(on and fades[leaf] or { leaf = leaf, enabled = false })
  end
end

-- Disarms the timer and turns the fades back on. Each part runs even when the
-- other fails.
local function rest()
  settling = false
  pcall(function()
    timer:set_enabled(false)
  end)
  pcall(set_fades, true)
end

-- Ends a strike lit. The fades stay off for one more step, so the lit colours
-- land at once, then `rest` turns them on; if the lit colours or that step
-- can't be had, it rests now, so no window is left dim or without its fades.
local function finish()
  if not step then
    return
  end
  step = nil
  local ok = pcall(paint, 1, group_at(1))
  settling = ok and pcall(function()
    timer:set_enabled(true)
  end)
  if not settling then
    rest()
  end
end

local function tick()
  if settling then
    rest()
    return
  end
  if not step then
    return
  end
  local ok = pcall(function()
    step = step + 1
    if step > #strike then
      finish()
    else
      paint(strike[step], group_at(strike[step]))
    end
  end)
  if not ok then
    finish()
  end
end

local function start(window)
  local ok = pcall(function()
    if not step and not settling then
      set_fades(false)
    end
    settling = false
    step = 1
    -- A grouped window's strike cut short by a plain one's leaves the group's
    -- colour dim: it goes back to lit with this first step.
    local was_grouped = grouped
    grouped = window.group ~= nil
    paint(strike[step], group_at(strike[step]) or (was_grouped and 1 or nil))
    if timer then
      timer:set_enabled(true)
    else
      timer = hl.timer(tick, { timeout = step_ms, type = "repeat" })
    end
  end)
  if not ok then
    step = step or 1
    finish()
  end
end

-- The stutter as the user layer's settings.json sets it: on unless a line
-- that isn't a `//` comment says `"focusStutter": false`, as
-- `config/quickshell/settings/Settings.qml` reads it. Read at each focus
-- change that would strike, so the Command menu's Item takes effect at the
-- next one: a read of the seed costs about 5 µs.
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
  start(window)
end

-- Sets an animation that a strike turns off and back on, the border's or the
-- shadow's fade: looks.lua's, or the user layer's over it.
function M.fade(spec)
  if not fades[spec.leaf] then
    leaves[#leaves + 1] = spec.leaf
  end
  fades[spec.leaf] = spec
  if not step and not settling then
    hl.animation(spec)
  end
end

-- Paints the active window's colours from palette `c`, at the strike's step
-- when one is running.
function M.use(c)
  palette = c
  local level = step and strike[step] or 1
  paint(level, group_at(level) or 1)
end

read_shown()
hl.on("window.active", on_active)
-- `workspace.move_to_monitor` changes what a screen shows without a
-- `workspace.active`: workspaces.lua moves them after a dock comes or goes.
for _, event in ipairs({
  "workspace.active",
  "workspace.special_active",
  "workspace.move_to_monitor",
  "monitor.added",
  "monitor.removed",
}) do
  hl.on(event, read_shown)
end

dot = dot or {}
dot.focus = M

return M
