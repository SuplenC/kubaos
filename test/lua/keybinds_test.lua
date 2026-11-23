-- The browser binds of `config/hypr/keybinds.lua` under plain Lua, against a
-- stand-in for `hl` and `dot` that takes any call: each bind's command run by
-- sh with stand-ins for Chrome and notify-send on PATH, which log their
-- arguments.
--
--   lua5.5 test/lua/keybinds_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/keybinds_test.lua REPO")
package.path = repo .. "/test/lua/?.lua;" .. package.path

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

-- Anything indexed or called gives itself back, so the file's other binds,
-- options and dispatchers load without a model of each.
local anything = setmetatable({}, {})
getmetatable(anything).__index = function() return anything end
getmetatable(anything).__call = function() return anything end

local binds = {}
hl = setmetatable({
  bind = function(keys, action, opts)
    binds[keys] = { action = action, description = opts.description }
  end,
  dsp = setmetatable({
    exec_cmd = function(cmd) return { cmd = cmd } end,
  }, { __index = anything }),
}, { __index = anything })
dot = { workspaces = setmetatable({ slots = 1 }, { __index = anything }) }

dofile(repo .. "/config/hypr/keybinds.lua")

local scratch <close> = require("scratch").new()
local dir = scratch:path()
local bin, log = dir .. "/bin", dir .. "/log"
os.execute("mkdir -p " .. bin)

local function stub(name)
  local file = assert(io.open(bin .. "/" .. name, "w"))
  file:write(("#!/bin/sh\n{ printf %%s %s; for a; do printf ' %%s' \"$a\"; done; echo; } >>'%s'\n"):format(name, log))
  file:close()
  os.execute("chmod +x " .. bin .. "/" .. name)
end

-- Runs a bind's command with only the stubs on PATH; gives back what they
-- logged.
local function run(keys)
  os.remove(log)
  local cmd = assert(binds[keys], keys .. " is bound").action.cmd
  os.execute(("PATH='%s' /bin/sh -c '%s'"):format(bin, (cmd:gsub("'", "'\\''"))))
  local file = io.open(log, "r")
  if not file then return "" end
  local out = file:read("a")
  file:close()
  return out
end

stub("notify-send")

local TOAST = "notify-send -a Browser -i web-browser Google Chrome isn't installed"
  .. " Install it from the Command menu (Install Google Chrome) or with kubaos app add google-chrome.\n"

test("without Chrome, both browser binds say how to install it", function()
  same(run("SUPER + SHIFT + B"), TOAST, "SUPER + SHIFT + B")
  same(run("SUPER + SHIFT + Return"), TOAST, "SUPER + SHIFT + Return")
  same(run("SUPER + SHIFT + ALT + B"), TOAST, "SUPER + SHIFT + ALT + B")
end)

stub("google-chrome-stable")

test("with Chrome, the browser binds start it and nothing else", function()
  same(run("SUPER + SHIFT + B"), "google-chrome-stable\n", "SUPER + SHIFT + B")
  same(run("SUPER + SHIFT + Return"), "google-chrome-stable\n", "SUPER + SHIFT + Return")
  same(run("SUPER + SHIFT + ALT + B"), "google-chrome-stable --incognito\n", "SUPER + SHIFT + ALT + B")
end)

os.exit(failed == 0 and 0 or 1, true)
