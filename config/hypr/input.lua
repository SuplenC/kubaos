-- Keyboard and touchpad.
-- https://wiki.hypr.land/Configuring/Variables/#input

-- The keyboard layouts are the system's: /etc/default/keyboard, which the
-- install writes from Kōbō's Language screen and `kubaos keyboard` rewrites
-- (through its root helper, system/files/keyboard). Several layouts are XKB's
-- comma-separated lists (XKBLAYOUT="us,it", XKBVARIANT=",intl"), which
-- Hyprland takes as they are, the first the default; SUPER + SHIFT + Space
-- switches (keybinds.lua). Read on every config load, so after a change
-- `hyprctl reload` picks it up. Without the file (a hand-install that never
-- set one), or without a layout in it, the US layout (`us`).
-- Workspace and other number/punctuation binds use keycodes (code:10 = the
-- "1" key), so they work whatever the layout is.
local function system_keyboard(path)
  local kb = { layout = "us", variant = "", options = "" }
  local file = io.open(path, "r")
  if not file then
    return kb
  end
  local found = {}
  for line in file:lines() do
    -- Shell syntax: KEY="value", KEY='value' or KEY=value.
    local key, value = line:match("^%s*(XKB%u+)%s*=%s*(.-)%s*$")
    if key then
      value = value:match('^"(.*)"$') or value:match("^'(.*)'$") or value
      found[key] = value
    end
  end
  file:close()
  if found.XKBLAYOUT and found.XKBLAYOUT ~= "" then
    kb.layout = found.XKBLAYOUT
    kb.variant = found.XKBVARIANT or ""
    kb.options = found.XKBOPTIONS or ""
  end
  return kb
end

-- The Mac-style accents on Left Alt: after Left Alt + e, `, u, i or n, keyd
-- sends a dead key on a spare keycode, and this XKB option gives those
-- keycodes their symbols in every layout (system/files/xkb/symbols/kubaos),
-- so the app composes the accent with the layout's own letter. It joins the options of
-- /etc/default/keyboard, never replaces them, and only where
-- system/50-hyprland.sh installed it; elsewhere libxkbcommon would log an
-- unknown option on every load. It stays while `kubaos keyboard accents off`
-- has stopped keyd: nothing sends those keycodes then.
local MAC_ACCENTS = "kubaos:mac-accents"

local function with_mac_accents(options, symbols)
  local file = io.open(symbols, "r")
  if not file then
    return options
  end
  file:close()
  for option in options:gmatch("[^,]+") do
    if option == MAC_ACCENTS then
      return options
    end
  end
  return options == "" and MAC_ACCENTS or options .. "," .. MAC_ACCENTS
end

local kb = system_keyboard("/etc/default/keyboard")
kb.options = with_mac_accents(kb.options, "/etc/xkb/symbols/kubaos")

hl.config({
  input = {
    kb_layout = kb.layout,
    kb_variant = kb.variant,
    kb_options = kb.options,

    repeat_rate = 40,   -- chars per second once repeating
    repeat_delay = 250, -- ms before repeat starts

    follow_mouse = 1,
    sensitivity = 0,

    touchpad = {
      natural_scroll = true,
      tap_to_click = true,
      disable_while_typing = true,
      clickfinger_behavior = true, -- two-finger click = right click, like macOS
      scroll_factor = 0.5,
    },
  },

  cursor = {
    hide_on_key_press = true,
  },

  binds = {
    hide_special_on_workspace_change = true,
  },
})

-- The three-finger swipe through the workspaces, and its gesture settings, are
-- in keybinds.lua, beside the binds that walk the same workspaces.
