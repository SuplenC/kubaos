-- `config/hypr/launches.lua` under plain Lua, against a stand-in for `hl`:
-- each launch's window rule, when it is on, and the windows it places.
--
--   lua5.5 test/lua/launches_test.lua REPO
--
-- A window "of the launch's unit" is given this process's own pid, and the
-- launch the unit this process runs in, so the module reads a real
-- `/proc/<pid>/cgroup`. Prints an ok or FAIL line per test and exits 1 when
-- any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/launches_test.lua REPO")
local module = repo .. "/config/hypr/launches.lua"

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

local function read_line(path)
  local file = assert(io.open(path, "r"))
  local line = file:read("l")
  file:close()
  return line
end

local OWN_PID = tonumber(read_line("/proc/self/stat"):match("^(%d+)"))
local OWN_UNIT = read_line("/proc/self/cgroup"):match("([^/]+)$")

-- launches.lua loaded fresh against a stand-in for `hl`, with workspaces 1
-- and 2 on two screens, 1 focused, and window `A` active on it.
local function world()
  local w = {
    workspaces = { [1] = { id = 1 }, [2] = { id = 2 } },
    active = 1,
    active_window = { address = "0xa" },
    rules = {},
    timers = {},
    handlers = {},
    dispatched = {},
  }

  local function rule_handle(spec)
    local rule = { spec = spec, enabled = true }
    function rule.set_enabled(self, on)
      self.enabled = on
    end
    return rule
  end

  w.hl = {
    get_active_workspace = function()
      return w.workspaces[w.active]
    end,
    get_workspace = function(id)
      return w.workspaces[id]
    end,
    get_active_window = function()
      return w.active_window
    end,
    get_active_monitor = function()
      return { name = "DP-" .. w.active }
    end,
    window_rule = function(spec)
      local rule = rule_handle(spec)
      w.rules[spec.name] = rule
      return rule
    end,
    timer = function(fn, opts)
      local timer = { fn = fn, opts = opts, enabled = true }
      function timer.set_enabled(self, on)
        self.enabled = on
      end
      w.timers[#w.timers + 1] = timer
      return timer
    end,
    on = function(event, fn)
      w.handlers[event] = w.handlers[event] or {}
      table.insert(w.handlers[event], fn)
    end,
    dispatch = function(d)
      w.dispatched[#w.dispatched + 1] = d
    end,
    dsp = {
      focus = function(args)
        return { what = "focus", args = args }
      end,
      window = {
        move = function(args)
          return { what = "move", args = args }
        end,
      },
    },
  }

  function w.fire(event, ...)
    for _, fn in ipairs(w.handlers[event] or {}) do
      fn(...)
    end
  end

  -- Focus moves to workspace `id`, as `workspace.active` tells.
  function w.focus(id)
    w.active = id
    w.fire("workspace.active", w.workspaces[id])
  end

  function w.remove(id)
    w.workspaces[id] = nil
    w.fire("workspace.removed", { id = id })
  end

  -- A window maps on the focused workspace, unless an enabled rule matching
  -- its class sends it elsewhere, as Hyprland applies `workspace <id> silent`.
  function w.open(window)
    window.workspace = w.workspaces[w.active]
    for _, rule in pairs(w.rules) do
      if rule.enabled and window.class:lower():match(w.pattern(rule.spec.match.class)) then
        window.workspace = w.workspaces[tonumber(rule.spec.workspace:match("^(%d+) silent$"))]
      end
    end
    w.fire("window.open_early", window)
    w.fire("window.open", window)
    return window
  end

  -- The rule's `(?i)^…$` regex as a Lua pattern, for the stand-in's matching.
  function w.pattern(regex)
    local body = assert(regex:match("^%(%?i%)%^(.*)%$$"), regex)
    return "^" .. body:lower():gsub("\\(.)", "%%%1") .. "$"
  end

  _G.hl = w.hl
  _G.dot = {}
  w.launches = dofile(module)
  return w
end

local function only_rule(w)
  local name, rule = next(w.rules)
  assert(name and next(w.rules, name) == nil, "one rule")
  return rule
end

test("a launch's rule sends its app's windows to its workspace, silently", function()
  local w = world()

  w.launches.started([[app-kotodama-google\x2dchrome@1.service]], "google-chrome")

  local rule = only_rule(w)
  same(rule.spec.name, "launch-1", "name")
  same(rule.spec.match.class, [[(?i)^google\-chrome$]], "class")
  same(rule.spec.workspace, "1 silent", "workspace")
end)

test("the rule is off while the launch's workspace is the focused one", function()
  local w = world()

  w.launches.started("app-kotodama-foot@1.service", "foot")
  local stayed = only_rule(w).enabled
  w.focus(2)
  local moved = only_rule(w).enabled
  w.focus(1)

  same(stayed, false, "while on the launch's workspace")
  same(moved, true, "once on another")
  same(only_rule(w).enabled, false, "back on it")
end)

test("a window of the app opens on the launch's workspace and ends the launch", function()
  local w = world()

  w.launches.started([[app-kotodama-google\x2dchrome@1.service]], "google-chrome")
  w.focus(2)
  local window = w.open({ address = "0xc", class = "google-chrome", pid = 0 })

  same(window.workspace.id, 1, "workspace")
  same(#w.dispatched, 0, "dispatches")
  same(only_rule(w).enabled, false, "rule after")
  same(w.timers[1].enabled, false, "timer after")
  local later = w.open({ address = "0xd", class = "google-chrome", pid = 0 })
  same(later.workspace.id, 2, "a later window")
end)

test("a window whose class isn't the app id is moved and the focus handed back", function()
  local w = world()

  w.launches.started(OWN_UNIT, "steam")
  w.focus(2)
  w.active_window = { address = "0xb" }
  w.open({ address = "0xc", class = "steamwebhelper", pid = OWN_PID })

  same(#w.dispatched, 2, "dispatches")
  same(w.dispatched[1].what, "move", "first")
  same(w.dispatched[1].args.workspace, "1", "to")
  same(w.dispatched[1].args.follow, false, "follow")
  same(w.dispatched[1].args.window, "address:0xc", "window")
  same(w.dispatched[2].what, "focus", "then")
  same(w.dispatched[2].args.window, "address:0xb", "back to")
end)

test("a window of the launch opens as any does when the user stayed", function()
  local w = world()

  w.launches.started(OWN_UNIT, "steam")
  local window = w.open({ address = "0xc", class = "steamwebhelper", pid = OWN_PID })

  same(window.workspace.id, 1, "workspace")
  same(#w.dispatched, 0, "dispatches")
end)

test("a launch whose workspace went away leaves its window where the user is", function()
  local w = world()

  w.launches.started("app-kotodama-foot@1.service", "foot")
  w.focus(2)
  w.remove(1)
  local window = w.open({ address = "0xc", class = "foot", pid = OWN_PID })

  same(window.workspace.id, 2, "workspace")
  same(#w.dispatched, 0, "dispatches")
end)

test("a launch ends when its time runs out", function()
  local w = world()

  w.launches.started("app-kotodama-foot@1.service", "foot")
  w.focus(2)
  local timer = w.timers[1]
  timer.fn()
  local window = w.open({ address = "0xc", class = "foot", pid = 0 })

  same(timer.opts.type, "oneshot", "timer")
  same(only_rule(w).enabled, false, "rule")
  same(window.workspace.id, 2, "workspace")
end)

test("launches take the rule names in turn, ending the oldest", function()
  local w = world()

  for n = 1, 9 do
    w.launches.started("app-kotodama-app" .. n .. "@1.service", "app" .. n)
  end

  local names = 0
  for _ in pairs(w.rules) do
    names = names + 1
  end
  same(names, 8, "rules")
  same(w.rules["launch-1"].spec.match.class, "(?i)^app9$", "launch-1")
  same(w.timers[1].enabled, false, "the first launch's timer")
end)

test("a launch from a special workspace places nothing", function()
  local w = world()
  w.workspaces[1].special = true

  w.launches.started("app-kotodama-foot@1.service", "foot")

  same(next(w.rules), nil, "rules")
end)

test("an app id's dots are matched literally", function()
  local w = world()

  w.launches.started("app-gnome-org.gnome.Nautilus-4021.scope", "org.gnome.Nautilus")

  same(only_rule(w).spec.match.class, [[(?i)^org\.gnome\.Nautilus$]], "class")
end)

os.exit(failed == 0 and 0 or 1)
