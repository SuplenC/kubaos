-- Keyboard, touchpad, gestures.
-- https://wiki.hypr.land/Configuring/Variables/#input

-- The keyboard layout is the system's: /etc/default/keyboard, which the
-- installer writes from its Keyboard screen and `localectl set-x11-keymap`
-- rewrites. Read on every config load, so after a change `hyprctl reload`
-- picks it up. Without the file (a hand-install that never set one), or
-- without a layout in it, the US layout (`us`).
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

local kb = system_keyboard("/etc/default/keyboard")

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

-- Three-finger horizontal swipe switches workspace.
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
