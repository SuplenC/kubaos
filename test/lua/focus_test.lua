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

-- Hyprland's focus reasons, as `window.active` passes them.
local FFM, KEYBIND, WORKSPACE_CHANGE, TOGGLE_SPECIAL, NEW_WINDOW = 1, 2, 11, 12, 16

local A = { address = "0xa", workspace = { id = 1 } }
local B = { address = "0xb", workspace = { id = 1 } }
local C = { address = "0xc", workspace = { id = 1 } }
local HIDDEN = { address = "0xd", workspace = { id = 2 } }

local SPECS = {
  border = { leaf = "border", enabled = true, speed = 3, bezier = "standard" },
  fadeShadow = { leaf = "fadeShadow", enabled = true, speed = 3, bezier = "effects" },
}

-- focus.lua loaded fresh against a stand-in for `hl`, with `A` focused on
-- workspace 1 and the user's settings.json holding `settings` (no file for
-- nil); through looks.lua when `looks` is true, with no generated palette.
-- The world records the config Hyprland holds and each call made.
local function world(settings, looks)
  local home = os.tmpname()
  os.remove(home)
  if settings then
    os.execute("mkdir -p '" .. home .. "/kubaos/desktop/quickshell'")
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

  local w = { handlers = {}, timers = {}, animations = {}, configs = 0, active = A, fail_config = false }
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
      w.held.group = group and group.border_active or w.held.group
      w.held.shadow = shadow and shadow.color or w.held.shadow
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
  w.palette = colors.defaults
  if looks then
    _G.dot = {}
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
    return w.animations.border == SPECS.border and w.animations.fadeShadow == SPECS.fadeShadow
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

test("a dark tube is an inactive window's border and shadow", function()
  local w = world(nil)

  local dark = at(w, 0)
  local inactive = w.focus.inactive(w.palette)

  same(dark.border, inactive.border, "border")
  same(dark.shadow, inactive.shadow, "shadow")
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
  local seen = { w.held.border }
  local fades_off = w.animations.border.enabled == false and w.animations.fadeShadow.enabled == false
  for _ = 2, #strike do
    w.tick()
    seen[#seen + 1] = w.held.border
  end
  w.tick()

  same(fades_off, true, "the border's and shadow's fades off while it strikes")
  same(w.timers[1].timeout, tokens.duration.medium // #strike, "step")
  same(w.timers[1].type, "repeat", "timer type")
  for i, level in ipairs(strike) do
    same(seen[i], at(w, level).border, "step " .. i)
  end
  same(w.lit(), true, "lit after the strike")
  same(w.armed(), false, "the timer disarmed")
  same(w.fades_on(), true, "the fades back as looks.lua set them")
end)

test("the glow strikes with the tube, and a grouped window's tube too", function()
  local w = world(nil)

  w.focus_on(B, FFM)
  w.tick()

  same(w.held.shadow, at(w, tokens.neon.strike[2]).shadow, "glow")
  same(w.held.group, at(w, tokens.neon.strike[2]).border, "group")
end)

test("a new focus mid-strike strikes again from the first step and still ends lit", function()
  local w = world(nil)
  local strike = tokens.neon.strike

  w.focus_on(B, KEYBIND)
  w.tick()
  w.tick()
  w.focus_on(C, KEYBIND)
  local first = w.held.border
  for _ = 1, #strike do
    w.tick()
  end

  same(first, at(w, strike[1]).border, "first step again")
  same(#w.timers, 1, "timers")
  same(w.lit(), true, "lit")
  same(w.armed(), false, "disarmed")
  same(w.fades_on(), true, "fades")
end)

test("a workspace switch mid-strike ends it lit at once", function()
  local w = world(nil)

  w.focus_on(B, KEYBIND)
  w.tick()
  w.focus_on(HIDDEN, WORKSPACE_CHANGE)

  same(w.lit(), true, "lit")
  same(w.armed(), false, "disarmed")
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

test("a step that fails ends the strike lit", function()
  local w = world(nil)

  w.focus_on(B, KEYBIND)
  w.fail_config = true
  w.tick()

  same(w.lit(), true, "lit")
  same(w.armed(), false, "disarmed")
  same(w.fades_on(), true, "fades")
end)

test("looks.lua lights the active window and keeps inactive ones neutral, and a strike ends as it set them", function()
  local w = world(nil, true)
  local inactive = w.focus.inactive(w.palette)
  local border, shadow = w.animations.border, w.animations.fadeShadow

  w.focus_on(B, KEYBIND)
  for _ = 1, #tokens.neon.strike do
    w.tick()
  end

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

test("reloading the colours mid-strike keeps the step, and the strike still ends lit", function()
  local w = world(nil, true)

  w.focus_on(B, KEYBIND)
  w.tick()
  _G.dot.reload_colors()
  local held = w.held.border
  for _ = 1, #tokens.neon.strike do
    w.tick()
  end

  same(held, at(w, tokens.neon.strike[2]).border, "the step through the reload")
  same(w.lit(), true, "lit")
  same(w.armed(), false, "disarmed")
end)

os.getenv = real_getenv
os.exit(failed == 0 and 0 or 1)
