-- `config/hypr/user_layer.lua` under plain Lua: the user layer's table of
-- apps agrees with the seeds in user-layer/, and a file's path follows
-- XDG_CONFIG_HOME.
--
--   lua5.5 test/lua/user_layer_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/user_layer_test.lua REPO")
package.path = repo .. "/config/hypr/?.lua;" .. package.path

local user_layer = require("user_layer")

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

local function with_env(vars, fn)
  local real_getenv = os.getenv
  os.getenv = function(name)
    if vars[name] ~= nil then
      return vars[name] or nil
    end
    return real_getenv(name)
  end
  local ok, err = pcall(fn)
  os.getenv = real_getenv
  assert(ok, err)
end

-- Every folder two levels down in user-layer/, as "<group>/<app>".
local function seed_folders()
  local found = {}
  local list = assert(io.popen("cd '" .. repo .. "/user-layer' && find . -mindepth 2 -maxdepth 2 -type d"))
  for line in list:lines() do
    found[#found + 1] = line:gsub("^%./", "")
  end
  list:close()
  return found
end

test("every seed folder is where the table puts its app", function()
  local folders = seed_folders()
  assert(#folders > 0, "no seed folders found")
  for _, folder in ipairs(folders) do
    local app = folder:match("[^/]+$")
    same(user_layer.apps[app], folder, "the table's folder for " .. app)
  end
end)

test("a file is in its app's folder under XDG_CONFIG_HOME", function()
  with_env({ XDG_CONFIG_HOME = "/cfg" }, function()
    same(user_layer.file("keyboard", "keyboard.toml"), "/cfg/kubaos/desktop/keyboard/keyboard.toml", "keyboard")
  end)
end)

test("without XDG_CONFIG_HOME the user layer is under ~/.config", function()
  with_env({ XDG_CONFIG_HOME = "", HOME = "/home/kuba" }, function()
    same(user_layer.file("hypr", "hyprland.lua"), "/home/kuba/.config/kubaos/desktop/hypr/hyprland.lua", "hypr")
  end)
end)

test("an app with no folder is an error, not a path", function()
  assert(not pcall(user_layer.file, "nothing", "x"), "a path for an unknown app")
end)

os.exit(failed == 0 and 0 or 1)
