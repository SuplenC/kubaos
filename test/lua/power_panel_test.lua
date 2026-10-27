-- `config/hypr/monitors.lua`'s power panel under plain Lua: the stop's refresh
-- rate and VRR land on the laptop's own rule, the user's included, keep its
-- position and scale, never reach another screen, and do nothing without the
-- power service's file or with the lid's panel off.
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

-- A runtime dir of its own, with the power service's file when `panel` is given.
local function runtime(panel, lid_off)
  local dir = os.tmpname()
  os.remove(dir)
  assert(os.execute("mkdir -p '" .. dir .. "/kubaos-power' '" .. dir .. "/dot-hypr'"))
  if panel then
    local file = assert(io.open(dir .. "/kubaos-power/panel", "w"))
    file:write(panel)
    file:close()
  end
  if lid_off then
    assert(io.open(dir .. "/dot-hypr/laptop-panel-off", "w")):close()
  end
  return dir
end

-- Loads monitors.lua as hyprland.lua does, then `user` (the user layer's
-- rules), then the power panel; returns the rules Hyprland got, in order.
local function load(dir, user)
  local applied = {}
  _G.hl = {
    monitor = function(spec)
      applied[#applied + 1] = spec
    end,
    get_monitor = function(name)
      if name == "eDP-1" then
        return { width = 2880, height = 1800, x = 4480, y = 240, scale = 1.5 }
      end
    end,
  }
  _G.dot = {}
  local real_getenv = os.getenv
  os.getenv = function(name)
    if name == "XDG_RUNTIME_DIR" then
      return dir
    end
    return real_getenv(name)
  end
  local ok, err = pcall(function()
    dofile(repo .. "/config/hypr/monitors.lua")
    if user then
      user()
    end
    dot.power_panel()
  end)
  os.getenv = real_getenv
  assert(ok, err)
  return applied
end

local function laptop_rules(applied)
  local found = {}
  for _, spec in ipairs(applied) do
    if spec.output == "eDP-1" then
      found[#found + 1] = spec
    end
  end
  return found
end

test("without the power service's file, nothing is added", function()
  local applied = load(runtime(nil))

  same(#applied, 1, "rules")
  same(applied[1].output, "", "the fallback rule")
end)

test("vrr on the fallback rule keeps its preferred mode, auto position and scale", function()
  local rules = laptop_rules(load(runtime("refresh=native\nvrr=1\n")))

  same(#rules, 1, "laptop rules")
  same(rules[1].mode, "preferred", "mode")
  same(rules[1].position, "auto", "position")
  same(rules[1].scale, "auto", "scale")
  same(rules[1].vrr, 1, "vrr")
end)

test("the user's own panel rule keeps its position and scale and takes the rate", function()
  local rules = laptop_rules(load(runtime("refresh=60\nvrr=0\n"), function()
    hl.monitor({ output = "eDP-1", mode = "2880x1800@120", position = "4480x240", scale = 1.5, bitdepth = 10 })
  end))

  local last = rules[#rules]
  same(last.mode, "2880x1800@60", "mode")
  same(last.position, "4480x240", "position")
  same(last.scale, 1.5, "scale")
  same(last.bitdepth, 10, "bitdepth")
  same(last.vrr, 0, "vrr")
end)

test("60 Hz on the fallback rule takes the panel's size as it runs", function()
  local rules = laptop_rules(load(runtime("refresh=60\nvrr=0\n")))

  same(rules[1].mode, "2880x1800@60", "mode")
end)

test("native on the user's rule keeps the user's mode", function()
  local rules = laptop_rules(load(runtime("refresh=native\nvrr=0\n"), function()
    hl.monitor({ output = "eDP-1", mode = "2880x1800@120", position = "4480x240", scale = 1.5 })
  end))

  same(rules[#rules].mode, "2880x1800@120", "mode")
end)

test("external screens are never given a power rule", function()
  local applied = load(runtime("refresh=60\nvrr=1\n"), function()
    hl.monitor({ output = "desc:LG Electronics LG ULTRAGEAR+ 510NTCZN7787", mode = "2560x1440@143.99Hz", position = "1920x0", scale = 1 })
  end)

  for _, spec in ipairs(applied) do
    if spec.output ~= "eDP-1" then
      same(spec.vrr, nil, spec.output .. " vrr")
    end
  end
  same(#laptop_rules(applied), 1, "laptop rules")
end)

test("a panel the lid turned off stays off", function()
  local rules = laptop_rules(load(runtime("refresh=60\nvrr=1\n", true)))

  same(#rules, 1, "laptop rules")
  same(rules[1].disabled, true, "disabled")
end)

test("a second load wraps hl.monitor once and forgets the last load's rules", function()
  local dir = runtime("refresh=native\nvrr=1\n")
  load(dir, function()
    hl.monitor({ output = "eDP-1", mode = "2880x1800@120", position = "0x0", scale = 1 })
  end)
  local rules = laptop_rules(load(dir))

  same(#rules, 1, "laptop rules")
  same(rules[1].position, "auto", "position")
end)

if failed > 0 then
  os.exit(1)
end
