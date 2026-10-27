-- `config/hypr/monitors.lua`'s power panel under plain Lua, against a model of
-- how Hyprland 0.56 applies monitor rules and VRR:
--
-- - `hl.monitor` merges into the rule of the same output name, and outside a
--   config load it schedules ensureMonitorStatus (LuaBindingsConfigRules.cpp,
--   hlMonitor).
-- - ensureMonitorStatus leaves a screen alone when its rule compares
--   FULL_MATCH with the one in use, and CMonitorRule::compare never looks at
--   `vrr` (MonitorRule.cpp): a mode, scale, bitdepth or enabled change is a
--   modeset; a position or transform change is a soft apply; VRR alone is
--   nothing.
-- - ensureVRR takes the in-use rule's `vrr`, else `misc:vrr`, and turns
--   adaptive sync on or off without a modeset (MonitorRuleManager.cpp);
--   setting `misc:vrr` runs it, and so does every config load and a screen
--   set up.
--
--   lua5.5 test/lua/power_panel_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/power_panel_test.lua REPO")

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

local function copy(t)
  local c = {}
  for k, v in pairs(t) do
    c[k] = v
  end
  return c
end

-- The fields CMonitorRule::compare reads, hard (a modeset) and soft.
local HARD = { "mode", "scale", "bitdepth", "disabled" }
local SOFT = { "position", "transform" }

local function compare(rule, active)
  for _, key in ipairs(HARD) do
    if rule[key] ~= active[key] then
      return "no_match"
    end
  end
  for _, key in ipairs(SOFT) do
    if rule[key] ~= active[key] then
      return "soft"
    end
  end
  return "full"
end

-- A Hyprland: its screens, the rules of the config loaded last, `misc:vrr`.
local function hyprland(dir)
  local H = { dir = dir, screens = {}, rules = {}, order = {}, misc_vrr = 0, parsing = false, signature = "sig1" }

  function H:screen(name, description)
    local screen = { name = name, description = description, active = nil, vrr_active = false, modesets = 0, soft = 0 }
    self.screens[#self.screens + 1] = screen
    return screen
  end

  function H:rule_for(screen)
    if self.rules[screen.name] then
      return self.rules[screen.name]
    end
    for _, output in ipairs(self.order) do
      if output:sub(1, 5) == "desc:" and screen.description:sub(1, #output - 5) == output:sub(6) then
        return self.rules[output]
      end
    end
    return self.rules[""] or {}
  end

  function H:ensure_status()
    for _, screen in ipairs(self.screens) do
      local rule = self:rule_for(screen)
      local cmp = compare(rule, screen.active)
      if cmp == "no_match" then
        screen.modesets = screen.modesets + 1
        screen.active = copy(rule)
      elseif cmp == "soft" then
        screen.soft = screen.soft + 1
        screen.active = copy(rule)
      end
    end
  end

  function H:ensure_vrr()
    for _, screen in ipairs(self.screens) do
      local use = screen.active.vrr
      if use == nil then
        use = self.misc_vrr
      end
      screen.vrr_active = use == 1
    end
  end

  -- Sets the screens up as Hyprland does once its config is loaded: a
  -- screen's first mode is its first modeset.
  function H:set_up()
    for _, screen in ipairs(self.screens) do
      screen.active = copy(self:rule_for(screen))
      screen.modesets = 1
    end
    self:ensure_vrr()
  end

  function H:api()
    local h = self
    return {
      monitor = function(spec)
        assert(spec.output, "hl.monitor needs an output")
        local rule = h.rules[spec.output] and copy(h.rules[spec.output]) or {}
        if not h.rules[spec.output] then
          h.order[#h.order + 1] = spec.output
        end
        for k, v in pairs(spec) do
          if k ~= "output" then
            rule[k] = v
          end
        end
        h.rules[spec.output] = rule
        if not h.parsing then
          h.scheduled = true
        end
      end,
      config = function(spec)
        if spec.misc and spec.misc.vrr ~= nil then
          h.misc_vrr = spec.misc.vrr
          if not h.parsing then
            h.vrr_scheduled = true
          end
        end
      end,
      get_monitors = function()
        local list = {}
        for _, screen in ipairs(h.screens) do
          if screen.active then
            list[#list + 1] = screen
          end
        end
        return list
      end,
      get_monitor = function(name)
        for _, screen in ipairs(h.screens) do
          if screen.name == name and screen.active then
            local w, hgt = (screen.active.mode or "2880x1800"):match("^(%d+)x(%d+)")
            return { width = tonumber(w) or 2880, height = tonumber(hgt) or 1800 }
          end
        end
      end,
    }
  end

  -- A config load: hyprland.lua's order, monitors.lua, the user layer, then
  -- the power panel. A reload of a running Hyprland applies what changed.
  function H:load(user)
    self.rules, self.order, self.parsing = {}, {}, true
    _G.hl = self:api()
    _G.dot = {}
    local real_getenv = os.getenv
    os.getenv = function(name)
      if name == "XDG_RUNTIME_DIR" then
        return self.dir
      end
      if name == "HYPRLAND_INSTANCE_SIGNATURE" then
        return self.signature
      end
      return real_getenv(name)
    end
    local ok, err = pcall(function()
      dofile(repo .. "/config/hypr/monitors.lua")
      if user then
        user()
      end
      pcall(dot.power_panel)
    end)
    os.getenv = real_getenv
    self.parsing = false
    assert(ok, err)
    local running = false
    for _, screen in ipairs(self.screens) do
      running = running or screen.active ~= nil
    end
    if running then
      self:ensure_status()
      self:ensure_vrr()
    else
      self:set_up()
    end
  end

  -- `hyprctl eval 'dot.power_panel()'`, then Hyprland's scheduled refresh.
  function H:eval()
    local real_getenv = os.getenv
    os.getenv = function(name)
      if name == "XDG_RUNTIME_DIR" then
        return self.dir
      end
      return real_getenv(name)
    end
    local ok, err = pcall(dot.power_panel)
    os.getenv = real_getenv
    assert(ok, err)
    if self.scheduled or self.vrr_scheduled then
      self:ensure_status()
      self:ensure_vrr()
    end
    self.scheduled, self.vrr_scheduled = false, false
  end

  return H
end

local function runtime()
  local dir = os.tmpname()
  os.remove(dir)
  assert(os.execute("mkdir -p '" .. dir .. "/kubaos-power' '" .. dir .. "/dot-hypr'"))
  return dir
end

local function stop(dir, text)
  local file = assert(io.open(dir .. "/kubaos-power/panel", "w"))
  file:write(text)
  file:close()
end

local KUBA = function()
  hl.monitor({ output = "desc:LG Electronics LG ULTRAGEAR+ 510NTCZN7787", mode = "2560x1440@143.99Hz", position = "1920x0", scale = 1 })
  hl.monitor({ output = "eDP-1", mode = "2880x1800@120", position = "4480x240", scale = 1.5 })
end

-- A login: a new Hyprland with the laptop panel and Kuba's LG on HDMI.
local function login(panel_text, user)
  local dir = runtime()
  if panel_text then
    stop(dir, panel_text)
  end
  local h = hyprland(dir)
  local laptop = h:screen("eDP-1", "China Star Optoelectronics Technology Co. Ltd MNE007ZA3-2")
  local lg = h:screen("HDMI-A-1", "LG Electronics LG ULTRAGEAR+ 510NTCZN7787")
  h:load(user)
  return h, laptop, lg
end

test("a rule's VRR alone never reaches a running screen, as in Hyprland", function()
  local h, laptop = login("refresh=native\nvrr=0\n")

  hl.monitor({ output = "eDP-1", vrr = 1 })
  h:ensure_status()
  h:ensure_vrr()

  same(laptop.vrr_active, false, "laptop VRR")
end)

test("the panel has VRR from the first frame of a login, before the service runs", function()
  local _, laptop, lg = login(nil, KUBA)

  same(laptop.vrr_active, true, "laptop VRR")
  same(lg.vrr_active, false, "external VRR")
end)

test("a stop turns VRR off and on at runtime with no modeset", function()
  local h, laptop, lg = login("refresh=native\nvrr=1\n", KUBA)

  stop(h.dir, "refresh=native\nvrr=0\n")
  h:eval()
  same(laptop.vrr_active, false, "VRR off")
  stop(h.dir, "refresh=native\nvrr=1\n")
  h:eval()

  same(laptop.vrr_active, true, "VRR on")
  same(laptop.modesets, 1, "modesets: the login's alone")
  same(lg.modesets, 1, "external modesets: the login's alone")
  same(lg.vrr_active, false, "external VRR")
end)

test("an external screen whose own rule asks for VRR keeps it whatever the stop", function()
  local h, _, lg = login("refresh=native\nvrr=1\n", function()
    hl.monitor({ output = "desc:LG Electronics LG ULTRAGEAR+ 510NTCZN7787", mode = "2560x1440@143.99Hz", vrr = 1 })
  end)

  stop(h.dir, "refresh=native\nvrr=0\n")
  h:eval()

  same(lg.vrr_active, true, "external VRR")
end)

test("60 Hz is one mode change of the panel alone, keeping the user's position and scale", function()
  local h, laptop, lg = login("refresh=native\nvrr=1\n", KUBA)

  stop(h.dir, "refresh=60\nvrr=0\n")
  h:eval()

  same(laptop.active.mode, "2880x1800@60", "mode")
  same(laptop.active.position, "4480x240", "position")
  same(laptop.active.scale, 1.5, "scale")
  same(laptop.vrr_active, false, "VRR")
  same(laptop.modesets, 2, "modesets")
  same(lg.modesets, 1, "external modesets")

  stop(h.dir, "refresh=native\nvrr=1\n")
  h:eval()
  same(laptop.active.mode, "2880x1800@120", "back to the user's mode")
  same(laptop.vrr_active, true, "VRR back")
end)

test("60 Hz on the default rule takes the panel's size", function()
  local h, laptop = login("refresh=native\nvrr=1\n")
  laptop.active.mode = "2880x1800"

  stop(h.dir, "refresh=60\nvrr=0\n")
  h:eval()

  same(laptop.active.mode, "2880x1800@60", "mode")
end)

test("a reload keeps the stop and changes nothing on screen", function()
  local h, laptop, lg = login("refresh=60\nvrr=0\n", KUBA)

  h:load(KUBA)

  same(laptop.active.mode, "2880x1800@60", "mode")
  same(laptop.vrr_active, false, "VRR")
  same(laptop.modesets, 1, "laptop modesets")
  same(lg.modesets, 1, "external modesets")
end)

test("screens set up before these rules keep VRR off until Hyprland starts again", function()
  local dir = runtime()
  stop(dir, "refresh=native\nvrr=1\n")
  local h = hyprland(dir)
  h.signature = "old-session"
  local laptop = h:screen("eDP-1", "China Star")
  local lg = h:screen("HDMI-A-1", "LG Electronics LG ULTRAGEAR+ 510NTCZN7787")
  laptop.active = { mode = "2880x1800@120", position = "4480x240", scale = 1.5 }
  lg.active = { mode = "2560x1440@143.99Hz", position = "1920x0", scale = 1 }

  h:load(KUBA)
  h:eval()

  same(lg.vrr_active, false, "external VRR")
  same(laptop.vrr_active, false, "laptop VRR, until the next login")
end)

test("a panel the lid turned off stays off", function()
  local dir = runtime()
  stop(dir, "refresh=60\nvrr=1\n")
  assert(io.open(dir .. "/dot-hypr/laptop-panel-off", "w")):close()
  local h = hyprland(dir)
  local laptop = h:screen("eDP-1", "China Star")

  h:load()

  same(laptop.active.disabled, true, "disabled")
end)

if failed > 0 then
  os.exit(1)
end
