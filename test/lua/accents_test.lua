-- `config/hypr/accents.lua` under plain Lua: the cheat sheet of the
-- Mac-style accents it hands Hyprland, for the Shortcuts View.
--
--   lua5.5 test/lua/accents_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/accents_test.lua REPO")
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

local real_open = io.open
local real_getenv = os.getenv

-- The submaps and binds accents.lua makes, with keyd's config there when
-- `installed` is true and the user's keyboard.toml holding `settings` (no
-- file for nil).
local function binds_for(installed, settings)
  local home = os.tmpname()
  os.remove(home)
  local settings_path = home .. "/kubaos/desktop/keyboard/keyboard.toml"
  io.open = function(name, mode)
    if name == "/etc/keyd/kubaos.conf" then
      return installed and real_open(repo .. "/system/files/keyd/kubaos.conf", mode) or nil
    end
    if name == settings_path then
      if not settings then
        return nil
      end
      local path = os.tmpname()
      local f = assert(real_open(path, "w"))
      f:write(settings)
      f:close()
      return real_open(path, mode)
    end
    return real_open(name, mode)
  end
  os.getenv = function(name)
    if name == "XDG_CONFIG_HOME" then
      return home
    end
    return real_getenv(name)
  end
  local seen = { submaps = {}, binds = {} }
  local submap
  _G.hl = {
    define_submap = function(name, fn)
      seen.submaps[#seen.submaps + 1] = name
      submap = name
      fn()
      submap = nil
    end,
    bind = function(keys, action, opts)
      seen.binds[#seen.binds + 1] = { keys = keys, action = action, description = opts.description, submap = submap }
    end,
    dsp = {
      exec_cmd = function(cmd)
        return cmd
      end,
    },
  }
  local ok, err = pcall(dofile, repo .. "/config/hypr/accents.lua")
  io.open = real_open
  os.getenv = real_getenv
  assert(ok, err)
  return seen
end

test("the five chords are binds in a submap nothing enters", function()
  local seen = binds_for(true, nil)
  same(#seen.submaps, 1, "submaps")
  same(seen.submaps[1], "mac-accents", "submap")
  same(#seen.binds, 5, "binds")
  local keys = {}
  for _, bind in ipairs(seen.binds) do
    same(bind.submap, "mac-accents", bind.keys .. "'s submap")
    keys[#keys + 1] = bind.keys
  end
  same(table.concat(keys, " "), "ALT + E ALT + grave ALT + U ALT + I ALT + N", "keys")
end)

test("each chord says the letter it makes and the mark alone after Space", function()
  local first = binds_for(true, nil).binds[1]
  same(first.description, "Mac accent: Left Alt + e, then a letter: é (acute); then Space: ´", "description")
  same(first.action, "notify-send 'Mac accents' '" .. first.description .. "'", "action")
end)

test("the grave's backtick stays in single quotes for the shell", function()
  local grave = binds_for(true, nil).binds[2]
  same(grave.action:find('"', 1, true), nil, "a double quote in the action")
  same(grave.description, "Mac accent: Left Alt + `, then a letter: à (grave); then Space: `", "description")
end)

test("the sheet shows with a commented seed and goes with the setting off", function()
  same(#binds_for(true, "# mac_accents = false\n").binds, 5, "binds with the seed")
  same(#binds_for(true, "mac_accents = true\n").binds, 5, "binds turned on")
  same(#binds_for(true, "mac_accents = false\n").submaps, 0, "submaps turned off")
  same(#binds_for(true, "  mac_accents = false # off\n").submaps, 0, "submaps off with a comment")
end)

test("without keyd's config there is no sheet", function()
  same(#binds_for(false, nil).submaps, 0, "submaps")
end)

os.exit(failed == 0 and 0 or 1)
