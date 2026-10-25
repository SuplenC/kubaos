-- The cheat sheet of the Mac-style accents on Left Alt, in the Shortcuts View
-- (SUPER + F1) and the Command menu's search. keyd types the accents, not
-- Hyprland (system/files/keyd/kubaos.conf); the View lists only Hyprland
-- binds, so each chord is a bind here, in a submap nothing ever enters. The
-- binds never fire, and ALT + E stays plain Alt to every app.
--
-- They show while the accents are on: keyd installed by
-- system/50-hyprland.sh, and no `mac_accents = false` in
-- ~/.config/kubaos/desktop/keyboard/keyboard.toml. `kubaos keyboard accents on|off`
-- reloads Hyprland, so the sheet follows the setting.

local user_layer = require("user_layer")

local KEYD_CONFIG = "/etc/keyd/kubaos.conf"
local SUBMAP = "mac-accents"

-- Hyprland's key name for each chord, the mark, a letter with it, and the
-- mark alone after Space, as keyd and the XKB symbols type them.
local ACCENTS = {
  { key = "E", mark = "acute", letter = "é", alone = "´" },
  { key = "grave", mark = "grave", letter = "à", alone = "`" },
  { key = "U", mark = "diaeresis", letter = "ö", alone = "¨" },
  { key = "I", mark = "circumflex", letter = "ô", alone = "ˆ" },
  { key = "N", mark = "tilde", letter = "ñ", alone = "˜" },
}

local function installed()
  local f = io.open(KEYD_CONFIG, "r")
  if f then
    f:close()
  end
  return f ~= nil
end

-- The setting as kubaos_core::keyboard::settings reads it: on unless the
-- file says `mac_accents = false`. A comment or a broken file leaves it on,
-- as the default does.
local function turned_off()
  local f = io.open(user_layer.file("keyboard", "keyboard.toml"), "r")
  if not f then
    return false
  end
  local off = false
  for line in f:lines() do
    local value = line:match("^%s*mac_accents%s*=%s*(%a+)")
    if value then
      off = value == "false"
    end
  end
  f:close()
  return off
end

if installed() and not turned_off() then
  hl.define_submap(SUBMAP, function()
    for _, accent in ipairs(ACCENTS) do
      local description = string.format(
        "Mac accent: Left Alt + %s, then a letter: %s (%s); then Space: %s",
        accent.key == "grave" and "`" or accent.key:lower(), accent.letter, accent.mark, accent.alone)
      -- Picked from the Command menu, the bind says how to type the accent.
      -- Single quotes, as the grave's ` would run a command in double ones.
      local tell = "notify-send 'Mac accents' '" .. description:gsub("'", "'\\''") .. "'"
      hl.bind("ALT + " .. accent.key, hl.dsp.exec_cmd(tell), { description = description })
    end
  end)
end
