-- The keyboard part of `config/hypr/input.lua` under plain Lua: what it
-- hands Hyprland for the layouts in /etc/default/keyboard.
--
--   lua5.5 test/lua/input_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/input_test.lua REPO")

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
-- (no file at all for nil).
local function input_for(text)
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

os.exit(failed == 0 and 0 or 1)
