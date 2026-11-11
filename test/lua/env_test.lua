-- `config/hypr/env.lua` under plain Lua, with a stand-in for `hl.env`: the
-- PATH Hyprland hands every program it starts.
--
--   lua5.5 test/lua/env_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/env_test.lua REPO")
package.path = repo .. "/config/hypr/?.lua;" .. package.path

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

-- The variables env.lua sets, run with `vars` in place of the environment's;
-- `false` unsets one.
local function session(vars)
  local real_getenv = os.getenv
  local set = {}
  os.getenv = function(name)
    if vars[name] ~= nil then
      return vars[name] or nil
    end
    return real_getenv(name)
  end
  _G.hl = {
    env = function(name, value)
      set[name] = value
    end,
  }
  local ok, err = pcall(dofile, repo .. "/config/hypr/env.lua")
  os.getenv = real_getenv
  _G.hl = nil
  assert(ok, err)
  return set
end

local HOME = "/home/kuba"

test("mise's shims come after ~/.local/bin and before Nix's tools", function()
  local env = session({ HOME = HOME, XDG_DATA_HOME = false, MISE_DATA_DIR = false, PATH = "/usr/bin:/bin" })
  same(
    env.PATH,
    "/home/kuba/.local/bin:/home/kuba/.local/share/mise/shims:/home/kuba/.nix-profile/bin:"
      .. "/nix/var/nix/profiles/default/bin:/usr/bin:/bin",
    "PATH"
  )
end)

test("the shims follow MISE_DATA_DIR, then XDG_DATA_HOME, as mise does", function()
  local moved = session({ HOME = HOME, XDG_DATA_HOME = "/data", MISE_DATA_DIR = "/m", PATH = "/usr/bin" })
  local xdg = session({ HOME = HOME, XDG_DATA_HOME = "/data", MISE_DATA_DIR = "", PATH = "/usr/bin" })
  same(moved.PATH:match("^[^:]*:([^:]*)"), "/m/shims", "with MISE_DATA_DIR")
  same(xdg.PATH:match("^[^:]*:([^:]*)"), "/data/mise/shims", "with XDG_DATA_HOME and an empty MISE_DATA_DIR")
end)

test("an entry the login shell already put on PATH is moved, not repeated", function()
  local env = session({
    HOME = HOME,
    XDG_DATA_HOME = false,
    MISE_DATA_DIR = false,
    PATH = "/home/kuba/go/bin:/home/kuba/.local/share/mise/shims:/usr/bin",
  })
  local _, shims = env.PATH:gsub("/mise/shims", "")
  same(shims, 1, "shims entries")
  same(env.PATH:match("/home/kuba/go/bin.*$"), "/home/kuba/go/bin:/usr/bin", "the rest, in its order")
end)

os.exit(failed == 0 and 0 or 1)
