-- How Hyprland's config starts and leaves the session, under plain Lua: the
-- startup apps and the shutdown hook (`config/hypr/autostart.lua`), and the
-- fallback exit bind (`config/hypr/fallback.lua`).
--
--   lua5.5 test/lua/session_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/session_test.lua REPO")

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

-- Loads FILE against a stand-in `hl` that keeps the binds by their keys
-- (an `exec_cmd` dispatcher is its command, any other is its name), the
-- event handlers by event, what `os.execute` was handed, and the commands
-- `hl.exec_cmd` starts once Hyprland has started.
local function load(file)
  local seen = { binds = {}, on = {}, executed = {}, started = {} }
  local function dispatcher(path)
    return setmetatable({}, {
      __index = function(_, key) return dispatcher(path == "" and key or path .. "." .. key) end,
      __call = function(_, arg) return path == "exec_cmd" and arg or path end,
    })
  end
  _G.hl = {
    dsp = dispatcher(""),
    bind = function(keys, action) seen.binds[keys] = action end,
    on = function(event, fn) seen.on[event] = fn end,
    exec_cmd = function(cmd) seen.started[#seen.started + 1] = cmd end,
  }
  local real_execute = os.execute
  os.execute = function(cmd) seen.executed[#seen.executed + 1] = cmd end
  local ok, err = pcall(function()
    dofile(repo .. "/config/hypr/" .. file)
    if seen.on["hyprland.start"] then
      seen.on["hyprland.start"]()
    end
    if seen.on["hyprland.shutdown"] then
      seen.on["hyprland.shutdown"]()
    end
  end)
  os.execute = real_execute
  assert(ok, err)
  return seen
end

test("the fallback exit stops graphical-session.target, waits, then exits Hyprland", function()
  local cmd = load("fallback.lua").binds["SUPER + SHIFT + CTRL + Escape"]
  same(type(cmd), "string", "the exit bind runs a command")
  local stop = cmd:find("systemctl --user stop graphical-session.target", 1, true)
  local exit = cmd:find("hyprctl dispatch 'hl.dsp.exit()'", 1, true)
  assert(stop, "the exit bind doesn't stop graphical-session.target: " .. cmd)
  assert(exit, "the exit bind doesn't exit Hyprland: " .. cmd)
  assert(stop < exit, "the exit bind exits Hyprland before the session's units stop: " .. cmd)
  same(cmd:find("--no-block", 1, true), nil, "--no-block in the exit bind, which must wait")
  assert(cmd:find("timeout %d+ systemctl"), "the stop before the exit has no time limit: " .. cmd)
end)

test("Hyprland's shutdown hook stops the session without waiting on it", function()
  local executed = load("autostart.lua").executed
  same(#executed, 1, "commands run on shutdown")
  same(executed[1], "systemctl --user --no-block stop graphical-session.target", "the shutdown hook")
end)

test("the startup apps start once the session is up, unless first-boot setup is pending", function()
  local started = load("autostart.lua").started
  local with_target
  for _, cmd in ipairs(started) do
    assert(not cmd:find("bitwarden", 1, true), "a command starts Bitwarden itself: " .. cmd)
    if cmd:find("start xdg-desktop-autostart.target", 1, true) then
      same(with_target, nil, "a second command starting the startup apps")
      with_target = cmd
    end
  end
  assert(with_target, "nothing starts xdg-desktop-autostart.target")
  local session = with_target:find("start hyprland-session.target", 1, true)
  local gate = with_target:find("kubaos/setup-done", 1, true)
  local target = with_target:find("start xdg-desktop-autostart.target", 1, true)
  assert(session and session < target, "the startup apps start before the session: " .. with_target)
  assert(gate and gate < target, "the startup apps don't wait for first-boot setup: " .. with_target)
end)

os.exit(failed == 0 and 0 or 1)
