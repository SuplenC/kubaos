-- The keyboard part of `config/hypr/input.lua` under plain Lua: what it
-- hands Hyprland for the layouts in the user's keyboard settings, else in
-- /etc/default/keyboard.
--
--   lua5.5 test/lua/input_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/input_test.lua REPO")
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

-- input.lua's `input` settings with /etc/default/keyboard holding `text`
-- (no file at all for nil), and the accents' XKB symbols installed when
-- `accents` is true, and the user's keyboard.toml holding `mine` (none for nil).
local function input_for(text, accents, mine)
  local path = os.tmpname()
  if text then
    local f = assert(real_open(path, "w"))
    f:write(text)
    f:close()
  else
    os.remove(path)
  end
  io.open = function(name, mode)
    if name == "/etc/default/keyboard" then
      return real_open(path, mode)
    end
    if name:match("/kubaos/desktop/keyboard/keyboard%.toml$") then
      if not mine then
        return nil
      end
      local user = os.tmpname()
      local f = assert(real_open(user, "w"))
      f:write(mine)
      f:close()
      local opened = real_open(user, mode)
      os.remove(user)
      return opened
    end
    if name == "/etc/xkb/symbols/kubaos" then
      return accents and real_open(repo .. "/system/files/xkb/symbols/kubaos", mode) or nil
    end
    return real_open(name, mode)
  end
  local seen
  _G.hl = setmetatable({
    config = function(settings)
      seen = settings.input or seen
    end,
  }, { __index = function() return function() end end })
  local ok, err = pcall(dofile, repo .. "/config/hypr/input.lua")
  io.open = real_open
  os.remove(path)
  assert(ok, err)
  return assert(seen, "input.lua set no input settings")
end

test("two layouts reach Hyprland as XKB's lists, the first the default", function()
  local input = input_for('XKBMODEL="pc105"\nXKBLAYOUT="us,it"\nXKBVARIANT=",intl"\nXKBOPTIONS=""\n')
  same(input.kb_layout, "us,it", "kb_layout")
  same(input.kb_variant, ",intl", "kb_variant")
  same(input.kb_options, "", "kb_options")
end)

test("one layout is a plain name", function()
  local input = input_for("XKBLAYOUT=de\nXKBVARIANT=nodeadkeys\n")
  same(input.kb_layout, "de", "kb_layout")
  same(input.kb_variant, "nodeadkeys", "kb_variant")
end)

test("without the file the layout is US", function()
  local input = input_for(nil)
  same(input.kb_layout, "us", "kb_layout")
  same(input.kb_variant, "", "kb_variant")
end)

test("the accents' XKB option follows the file's own options", function()
  local input = input_for('XKBLAYOUT="us,it"\nXKBOPTIONS="grp:alt_shift_toggle"\n', true)
  same(input.kb_options, "grp:alt_shift_toggle,kubaos:mac-accents", "kb_options")
end)

test("the accents' XKB option stands alone without other options, and once", function()
  same(input_for("XKBLAYOUT=us\n", true).kb_options, "kubaos:mac-accents", "kb_options")
  same(input_for(nil, true).kb_options, "kubaos:mac-accents", "kb_options without the file")
  local twice = input_for('XKBLAYOUT=us\nXKBOPTIONS="kubaos:mac-accents"\n', true)
  same(twice.kb_options, "kubaos:mac-accents", "kb_options already naming it")
end)

test("without the accents' XKB symbols the options are the file's", function()
  local input = input_for('XKBLAYOUT=us\nXKBOPTIONS="caps:escape"\n', false)
  same(input.kb_options, "caps:escape", "kb_options")
end)

test("the user's layouts win over the system's, whose options stay", function()
  local input = input_for('XKBLAYOUT="us"\nXKBOPTIONS="caps:escape"\n', false,
    '# Mine.\nmac_accents = false\nlayouts = ["de", "us(intl)", "it"]\n')
  same(input.kb_layout, "de,us,it", "kb_layout")
  same(input.kb_variant, ",intl,", "kb_variant")
  same(input.kb_options, "caps:escape", "kb_options")
end)

test("a user file without layouts leaves the system's", function()
  local input = input_for('XKBLAYOUT="us,it"\nXKBVARIANT=",intl"\n', false, "# layouts = [\"de\"]\n")
  same(input.kb_layout, "us,it", "kb_layout")
  same(input.kb_variant, ",intl", "kb_variant")
end)

os.exit(failed == 0 and 0 or 1)
