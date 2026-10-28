-- `config/hypr/focus.lua` under plain Lua, against a stand-in for `hl`: the
-- active window's tube and glow, and the strike it plays as focus arrives.
--
--   lua5.5 test/lua/focus_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/focus_test.lua REPO")
package.path = repo .. "/config/hypr/?.lua;" .. package.path

local tokens = dofile(repo .. "/config/hypr/tokens.lua")
local colors = dofile(repo .. "/config/hypr/colors.lua")

local failed = 0

local function test(name, fn)
  local ok, err = pcall(fn)
  if ok then
    print("ok   " .. name)
  else
    failed = failed + 1
    print("FAIL " .. name .. ": " .. tostring(err))
  end
end

local function same(actual, expected, what)
  if actual ~= expected then
    error(string.format("%s: got %q, want %q", what, tostring(actual), tostring(expected)), 2)
  end
end

local real_getenv = os.getenv
local scratch = os.tmpname()
os.remove(scratch)
local homes = 0

-- Hyprland's focus reasons, as `window.active` passes them.
local FFM, KEYBIND, WORKSPACE_CHANGE, TOGGLE_SPECIAL, NEW_WINDOW = 1, 2, 11, 12, 16

local A = { address = "0xa", workspace = { id = 1 } }
local B = { address = "0xb", workspace = { id = 1 } }
local C = { address = "0xc", workspace = { id = 1 } }
local HIDDEN = { address = "0xd", workspace = { id = 2 } }
local GROUPED = { address = "0xf", workspace = { id = 1 }, group = {} }

local SPECS = {
  border = { leaf = "border", enabled = true, speed = 3, bezier = "standard" },
  fadeShadow = { leaf = "fadeShadow", enabled = true, speed = 3, bezier = "effects" },
}

-- focus.lua loaded fresh against a stand-in for `hl`, with `A` focused on
-- workspace 1 and the user's settings.json holding `settings` (no file for
-- nil); through looks.lua when `looks` is true, with no generated palette.
-- The world records the config Hyprland holds and each call made.
local function world(settings, looks)
  homes = homes + 1
  local home = scratch .. "/" .. homes
  os.execute("mkdir -p '" .. home .. "/kubaos/desktop/quickshell'")
  if settings then
    local f = assert(io.open(home .. "/kubaos/desktop/quickshell/settings.json", "w"))
    f:write(settings)
    f:close()
  end
  -- The user layer stays this world's until the next one: focus.lua reads
  -- the settings at each focus change.
  os.getenv = function(name)
    if name == "XDG_CONFIG_HOME" or name == "HOME" then
      return home
    end
    return real_getenv(name)
  end

  local w = { handlers = {}, timers = {}, animations = {}, configs = 0, groups = 0, active = A, fail_config = false }
  w.held = { border = nil, group = nil, shadow = nil }
  -- Every key set, by its dotted name, as `hl.get_config` reads them.
  w.set = {}
  w.monitors = { { active_workspace = { id = 1 }, active_special_workspace = nil } }
  _G.hl = {
    config = function(t)
      if w.fail_config then
        w.fail_config = false
        error("hl.config failed")
      end
      w.configs = w.configs + 1
      local function walk(prefix, value)
        if type(value) == "table" and not value.colors then
          for key, inner in pairs(value) do
            walk(prefix == "" and key or prefix .. "." .. key, inner)
          end
        else
          w.set[prefix] = value
        end
      end
      walk("", t)
      local general = t.general and t.general.col
      local group = t.group and t.group.col
      local shadow = t.decoration and t.decoration.shadow
      w.held.border = general and general.active_border or w.held.border
      w.held.shadow = shadow and shadow.color or w.held.shadow
      if group and group.border_active then
        w.held.group = group.border_active
        w.groups = w.groups + 1
      end
    end,
    animation = function(spec)
      w.animations[spec.leaf] = spec
    end,
    curve = function() end,
    timer = function(callback, opts)
      local timer = { callback = callback, timeout = opts.timeout, type = opts.type, enabled = true }
      function timer.set_enabled(self, on)
        self.enabled = on
      end
      function timer.is_enabled(self)
        return self.enabled
      end
      w.timers[#w.timers + 1] = timer
      return timer
    end,
    on = function(event, fn)
      w.handlers[event] = w.handlers[event] or {}
      table.insert(w.handlers[event], fn)
    end,
    get_active_window = function()
      return w.active
    end,
    get_monitors = function()
      return w.monitors
    end,
  }

  package.loaded.focus = nil
  package.loaded.colors = nil
  _G.dot = {}
  w.palette = colors.defaults
  if looks then
    assert(pcall(dofile, repo .. "/config/hypr/looks.lua"))
    w.focus = package.loaded.focus
  else
    local ok, focus = pcall(require, "focus")
    assert(ok, focus)
    w.focus = focus
    w.focus.fade(SPECS.border)
    w.focus.fade(SPECS.fadeShadow)
    w.focus.use(w.palette)
  end
  w.configs = 0
  w.groups = 0

  function w.fire(event, ...)
    for _, fn in ipairs(w.handlers[event] or {}) do
      fn(...)
    end
  end

  -- Focus moving to `window`, as Hyprland moves it and then says so.
  function w.focus_on(window, reason)
    w.active = window
    w.fire("window.active", window, reason)
  end

  -- The repeating timers that are armed, fired once each.
  function w.tick()
    for _, timer in ipairs(w.timers) do
      if timer.enabled then
        timer.callback()
      end
    end
  end

  -- Ticks until no timer is armed, at most `limit` times; how many it took.
  function w.run(limit)
    local ticks = 0
    while w.armed() and ticks < (limit or 100) do
      w.tick()
      ticks = ticks + 1
    end
    return ticks
  end

  function w.armed()
    for _, timer in ipairs(w.timers) do
      if timer.enabled then
        return true
      end
    end
    return false
  end

  function w.lit()
    local lit = w.focus.colors(w.palette, 1)
    return w.held.border == lit.border and w.held.group == lit.border and w.held.shadow == lit.shadow
  end

  function w.fades_on()
    local border, shadow = w.animations.border, w.animations.fadeShadow
    return border.enabled ~= false and shadow.enabled ~= false
  end

  return w
end

local function at(w, level)
  return w.focus.colors(w.palette, level)
end

local function alpha(value)
  return string.format("%02x", math.floor(value * 255 + 0.5))
end

local function glow(d)
  local f = tokens.focus
  local function fade(reach, opacity)
    return d >= reach and 0 or opacity * (1 - d / reach) ^ 2
  end
  return 1 - (1 - fade(f.near, f.near_opacity)) * (1 - fade(f.far, f.far_opacity))
end

test("the lit tube is primary and its glow primary at the glow's opacity at the tube", function()
  local w = world(nil)

  local lit = at(w, 1)

  same(lit.border, colors.rgba(w.palette.primary, "ff"), "tube")
  same(lit.shadow, colors.rgba(w.palette.primary, alpha(glow(0))), "glow")
  same(w.lit(), true, "the active colours once the palette is in use")
end)

test("a dark tube is an inactive window's border and shadow, from the window opacities", function()
  local w = world(nil)

  local dark = at(w, 0)
  local inactive = w.focus.inactive(w.palette)

  same(dark.border, inactive.border, "border")
  same(dark.shadow, inactive.shadow, "shadow")
  same(inactive.border, colors.rgba(w.palette.surface_container_highest, alpha(tokens.opacity.window_border)), "border")
  same(inactive.shadow, colors.rgba(w.palette.shadow, alpha(tokens.opacity.window_shadow)), "shadow")
end)

test("the glow reaches as far as the far glow, falling off at the power that fits it best", function()
  local w = world(nil)

  same(w.focus.shadow.range, tokens.focus.far, "range")
  same(w.focus.shadow.power, 4, "power")
end)

test("focus moving to another window strikes the tube and glow through every step, then stays lit", function()
  local w = world(nil)
  local strike = tokens.neon.strike

  w.focus_on(B, KEYBIND)
  local seen = { { w.held.border, w.held.shadow } }
  local fades_off = not w.fades_on()
  for _ = 2, #strike do
    w.tick()
    seen[#seen + 1] = { w.held.border, w.held.shadow }
  end
  w.tick()
  local landed = { lit = w.lit(), fades_off = not w.fades_on(), armed = w.armed() }
  w.tick()

  same(fades_off, true, "the border's and shadow's fades off while it strikes")
  same(w.timers[1].timeout, tokens.duration.medium // #strike, "step")
  same(w.timers[1].type, "repeat", "timer type")
  for i, level in ipairs(strike) do
    same(seen[i][1], at(w, level).border, "tube at step " .. i)
    same(seen[i][2], at(w, level).shadow, "glow at step " .. i)
  end
  same(landed.lit and landed.fades_off and landed.armed, true, "lit with the fades still off for a step")
  same(w.lit(), true, "lit after the strike")
  same(w.armed(), false, "the timer disarmed")
  same(w.animations.border, SPECS.border, "the border's fade as set")
  same(w.animations.fadeShadow, SPECS.fadeShadow, "the shadow's fade as set")
end)

test("only a grouped window's strike sets the group's colour", function()
  local plain = world(nil)
  plain.focus_on(B, FFM)
  plain.run()
  same(plain.groups, 0, "group colour sets for a plain window")

  local w = world(nil)
  w.focus_on(GROUPED, FFM)
  w.tick()
  same(w.held.group, at(w, tokens.neon.strike[2]).border, "group tube at step 2")
  w.run()
  same(w.lit(), true, "lit")
end)

test("a grouped window's strike cut short by a plain window's leaves the group's colour lit", function()
  local w = world(nil)

  w.focus_on(GROUPED, KEYBIND)
  w.tick()
  w.focus_on(B, KEYBIND)
  w.run()

  same(w.lit(), true, "lit, the group's colour too")
end)

test("a new focus mid-strike strikes again from the first step and still ends lit", function()
  local w = world(nil)
  local strike = tokens.neon.strike

  w.focus_on(B, KEYBIND)
  w.tick()
  w.tick()
  w.focus_on(C, KEYBIND)
  local first = w.held.border
  local ticks = w.run()

  same(first, at(w, strike[1]).border, "first step again")
  same(ticks, #strike + 1, "ticks from the new focus")
  same(#w.timers, 1, "timers")
  same(w.lit(), true, "lit")
  same(w.fades_on(), true, "fades")
end)

test("a new focus while the lit colours land strikes again with the fades still off", function()
  local w = world(nil)

  w.focus_on(B, KEYBIND)
  for _ = 1, #tokens.neon.strike do
    w.tick()
  end
  w.focus_on(C, KEYBIND)
  local striking = { border = w.held.border, fades_off = not w.fades_on() }
  w.run()

  same(striking.border, at(w, tokens.neon.strike[1]).border, "first step")
  same(striking.fades_off, true, "fades off")
  same(w.lit() and w.fades_on() and not w.armed(), true, "lit, fades on, disarmed")
end)

test("a workspace switch mid-strike lights the window at once and then turns the fades on", function()
  local w = world(nil)

  w.focus_on(B, KEYBIND)
  w.tick()
  w.focus_on(HIDDEN, WORKSPACE_CHANGE)
  local lit, fades_off = w.lit(), not w.fades_on()
  local ticks = w.run()

  same(lit, true, "lit at once")
  same(fades_off, true, "with the fades off as it lands")
  same(ticks, 1, "ticks to the fades")
  same(w.fades_on(), true, "fades")
end)

test("with the stutter off nothing strikes and Hyprland's own fade lights the window", function()
  local w = world('{\n  "focusStutter": false\n}\n')

  w.focus_on(B, KEYBIND)

  same(w.configs, 0, "config calls")
  same(#w.timers, 0, "timers")
  same(w.fades_on(), true, "fades")
end)

test("the stutter is on by default, and a commented-out line leaves it on", function()
  local commented = world('{\n  // "focusStutter": false\n}\n')
  commented.focus_on(B, KEYBIND)
  same(commented.armed(), true, "a commented line")

  local seed = world(nil)
  seed.focus_on(B, KEYBIND)
  same(seed.armed(), true, "no file")

  local on = world('{\n  "menuMotion": "fades",\n  "focusStutter": true\n}\n')
  on.focus_on(B, KEYBIND)
  same(on.armed(), true, "set on")
end)

test("focus coming back to the same window doesn't strike: an unlock, a closed menu", function()
  local w = world(nil)

  w.focus_on(nil, FFM)
  w.focus_on(A, FFM)

  same(w.configs, 0, "config calls")
  same(#w.timers, 0, "timers")
end)

test("a workspace switch, a special workspace or a new window doesn't strike", function()
  for _, reason in ipairs({ WORKSPACE_CHANGE, TOGGLE_SPECIAL, NEW_WINDOW }) do
    local w = world(nil)

    w.focus_on(B, reason)

    same(#w.timers, 0, "timers for reason " .. reason)
    same(w.configs, 0, "config calls for reason " .. reason)
  end
end)

test("the pointer landing on a window of a workspace just switched to doesn't strike", function()
  local w = world(nil)

  w.monitors[1].active_workspace = { id = 2 }
  w.focus_on(HIDDEN, FFM)
  w.fire("workspace.active", { id = 2 })
  local during = #w.timers
  w.focus_on({ address = "0xe", workspace = { id = 2 } }, FFM)

  same(during, 0, "timers during the switch")
  same(w.armed(), true, "a strike once the workspace shows")
end)

test("a workspace moved to a screen strikes once it shows there", function()
  local w = world(nil)

  w.monitors[1].active_workspace = { id = 2 }
  w.fire("workspace.move_to_monitor", { id = 2 }, w.monitors[1])
  w.focus_on(HIDDEN, KEYBIND)

  same(w.armed(), true, "a strike on the moved workspace")
end)

test("a palette change mid-strike paints the steps left in the new colours", function()
  local w = world(nil)
  local light = {}
  for key, value in pairs(colors.defaults) do
    light[key] = value
  end
  light.primary = "#8e4e00"

  w.focus_on(B, KEYBIND)
  w.palette = light
  w.focus.use(light)
  w.tick()

  same(w.held.border, at(w, tokens.neon.strike[2]).border, "step in the new palette")
end)

test("a step that fails ends the strike lit, with the fades back", function()
  local w = world(nil)

  w.focus_on(B, KEYBIND)
  w.fail_config = true
  w.tick()
  w.run()

  same(w.lit(), true, "lit")
  same(w.armed(), false, "disarmed")
  same(w.fades_on(), true, "fades")
end)

test("a strike that can't start leaves the window lit with its fades", function()
  local w = world(nil)

  w.fail_config = true
  w.focus_on(B, KEYBIND)
  w.run()

  same(w.lit(), true, "lit")
  same(w.armed(), false, "disarmed")
  same(w.fades_on(), true, "fades")
end)

test("looks.lua lights the active window and keeps inactive ones neutral, and a strike ends as it set them", function()
  local w = world(nil, true)
  local inactive = w.focus.inactive(w.palette)
  local border, shadow = w.animations.border, w.animations.fadeShadow

  w.focus_on(B, KEYBIND)
  w.run()

  same(w.lit(), true, "lit")
  same(w.set["general.col.inactive_border"], inactive.border, "inactive border")
  same(w.set["group.col.border_inactive"], inactive.border, "inactive group border")
  same(w.set["decoration.shadow.color_inactive"], inactive.shadow, "inactive shadow")
  same(w.set["decoration.shadow.range"], w.focus.shadow.range, "range")
  same(w.set["decoration.shadow.render_power"], w.focus.shadow.power, "power")
  same(w.animations.border, border, "border fade")
  same(w.animations.fadeShadow, shadow, "shadow fade")
  same(border.enabled and border.speed * 100, tokens.duration.medium, "border fade's length")
end)

test("a fade the user layer sets through dot.focus.fade is the one a strike brings back", function()
  local w = world(nil, true)
  local mine = { leaf = "border", enabled = true, speed = 1, bezier = "standard" }

  _G.dot.focus.fade(mine)
  w.focus_on(B, KEYBIND)
  w.run()

  same(w.animations.border, mine, "border fade")
end)

test("reloading the colours mid-strike keeps the step, and the strike still ends lit", function()
  local w = world(nil, true)

  w.focus_on(B, KEYBIND)
  w.tick()
  _G.dot.reload_colors()
  local held = w.held.border
  w.run()

  same(held, at(w, tokens.neon.strike[2]).border, "the step through the reload")
  same(w.lit(), true, "lit")
  same(w.armed(), false, "disarmed")
end)

os.getenv = real_getenv
os.execute("rm -rf '" .. scratch .. "'")
os.exit(failed == 0 and 0 or 1)
