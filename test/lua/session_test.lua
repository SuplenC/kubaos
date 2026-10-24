-- How Hyprland's config leaves the session, under plain Lua: the fallback
-- exit bind (`config/hypr/fallback.lua`) and the shutdown hook
-- (`config/hypr/autostart.lua`).
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
-- event handlers by event, and what `os.execute` was handed.
local function load(file)
  local seen = { binds = {}, on = {}, executed = {} }
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
    exec_cmd = function() end,
  }
  local real_execute = os.execute
  os.execute = function(cmd) seen.executed[#seen.executed + 1] = cmd end
  local ok, err = pcall(function()
    dofile(repo .. "/config/hypr/" .. file)
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

os.exit(failed == 0 and 0 or 1)
