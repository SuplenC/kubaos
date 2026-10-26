-- Keybinds, ported from Omarchy 4 ("Quattro", default/hypr/bindings/*.lua)
-- with window focus on hjkl (docs/CONTRACTS.md) and apps adapted to this
-- setup. Each description is the bind's title in the Command menu, which
-- finds it by typing what it does; SUPER + F1 lists them all in its Shortcuts
-- View. The terminal, the menu and the survival binds live in fallback.lua.
--
-- Number and punctuation keys use keycodes (code:10 = "1" ... code:19 = "0",
-- code:20/21 = the two keys right of "0", code:49 = the key left of "1"),
-- so the binds sit on the same physical keys with the Italian layout.

local function bind(keys, description, action, flags)
  local opts = { description = description }
  for k, v in pairs(flags or {}) do
    opts[k] = v
  end
  if type(action) == "string" then
    action = hl.dsp.exec_cmd(action)
  end
  return hl.bind(keys, action, opts)
end

local locked = { locked = true }
local locked_repeat = { locked = true, repeating = true }

local browser = "google-chrome-stable"
-- A window of Ghostty's background instance running cmd (home/terminal.nix),
-- or a Ghostty of its own when the instance can't be reached. $HOME, as in
-- fallback.lua's SUPER + Return.
local function tui(cmd)
  return 'ghostty +new-window --working-directory="$HOME" -e ' .. cmd .. " || exec ghostty -e " .. cmd
end

---------------------------------------------------------------------------
-- Applications
---------------------------------------------------------------------------

bind("SUPER + SHIFT + Return", "Browser", browser)
bind("SUPER + SHIFT + B", "Browser", browser)
bind("SUPER + SHIFT + ALT + B", "Browser (incognito)", browser .. " --incognito")
bind("SUPER + SHIFT + F", "File manager (Files)", "nautilus --new-window")
bind("SUPER + CTRL + F", "File manager (yazi)", "kubaos-files open")
bind("SUPER + SHIFT + N", "Editor (nvim)", tui("nvim"))
bind("SUPER + CTRL + Return", "Terminal in tmux", tui("tmux new-session -A -s main"))
bind("SUPER + SHIFT + D", "Docker (lazydocker)", tui("lazydocker"))
bind("SUPER + CTRL + T", "Activity (btop)", tui("btop"))
bind("SUPER + F1", "Keyboard shortcuts", "kotodama open --view hyprland/shortcuts")

---------------------------------------------------------------------------
-- Windows
---------------------------------------------------------------------------

bind("SUPER + Q", "Close window", hl.dsp.window.close())
bind("CTRL + ALT + Delete", "Close all windows", function()
  for _, w in ipairs(hl.get_windows()) do
    hl.dispatch(hl.dsp.window.close({ window = "address:" .. w.address }))
  end
end)

bind("SUPER + E", "Toggle split direction", hl.dsp.layout("togglesplit"))
bind("SUPER + P", "Pseudo-tile window", hl.dsp.window.pseudo())
bind("SUPER + T", "Toggle floating", hl.dsp.window.float({ action = "toggle" }))
bind("SUPER + F", "Fullscreen", hl.dsp.window.fullscreen({ mode = "fullscreen" }))
bind("SUPER + ALT + F", "Maximize (keep gaps and bar)", hl.dsp.window.fullscreen({ mode = "maximized" }))

-- Pop out: float + pin + center (again to put it back).
bind("SUPER + O", "Pop window out (float and pin)", function()
  local w = hl.get_active_window()
  if not w then return end
  if w.floating and w.pinned then
    hl.dispatch(hl.dsp.window.pin({ action = "disable" }))
    hl.dispatch(hl.dsp.window.float({ action = "disable" }))
  else
    if not w.floating then
      hl.dispatch(hl.dsp.window.float({ action = "enable" }))
    end
    hl.dispatch(hl.dsp.window.resize({ x = 1100, y = 700 }))
    hl.dispatch(hl.dsp.window.center())
    hl.dispatch(hl.dsp.window.pin({ action = "enable" }))
  end
end)

-- Windows on hjkl, screens on the arrows (docs/CONTRACTS.md): focus, move
-- (SHIFT) and resize (CTRL) all act on the window under the same keys.
local dirs = {
  { key = "H", arrow = "Left", dir = "l", dx = -50, dy = 0, name = "left" },
  { key = "J", arrow = "Down", dir = "d", dx = 0, dy = 50, name = "down" },
  { key = "K", arrow = "Up", dir = "u", dx = 0, dy = -50, name = "up" },
  { key = "L", arrow = "Right", dir = "r", dx = 50, dy = 0, name = "right" },
}
for _, d in ipairs(dirs) do
  bind("SUPER + " .. d.key, "Focus window " .. d.name, hl.dsp.focus({ direction = d.dir }))
  bind("SUPER + " .. d.arrow, "Focus screen " .. d.name, hl.dsp.focus({ monitor = d.dir }))
  bind("SUPER + SHIFT + " .. d.key, "Move window " .. d.name, hl.dsp.window.move({ direction = d.dir }))
  bind("SUPER + SHIFT + " .. d.arrow, "Swap window " .. d.name, hl.dsp.window.swap({ direction = d.dir }))
  bind("SUPER + CTRL + " .. d.key, "Resize window " .. d.name,
    hl.dsp.window.resize({ x = d.dx, y = d.dy, relative = true }), { repeating = true })
  bind("SUPER + SHIFT + ALT + " .. d.key, "Move workspace to screen " .. d.name, dot.workspaces.move_to_monitor(d.dir))
  bind("SUPER + SHIFT + ALT + " .. d.arrow, "Move workspace to screen " .. d.name, dot.workspaces.move_to_monitor(d.dir))
  bind("SUPER + ALT + " .. d.arrow, "Move window into group " .. d.name, hl.dsp.window.move({ into_group = d.dir }))
end

-- Omarchy's width/height keys (the two keys right of 0).
bind("SUPER + code:20", "Shrink width", hl.dsp.window.resize({ x = -100, y = 0, relative = true }), { repeating = true })
bind("SUPER + code:21", "Grow width", hl.dsp.window.resize({ x = 100, y = 0, relative = true }), { repeating = true })
bind("SUPER + SHIFT + code:20", "Shrink height", hl.dsp.window.resize({ x = 0, y = -100, relative = true }), { repeating = true })
bind("SUPER + SHIFT + code:21", "Grow height", hl.dsp.window.resize({ x = 0, y = 100, relative = true }), { repeating = true })

-- Mouse.
bind("SUPER + mouse:272", "Drag window", hl.dsp.window.drag(), { mouse = true })
bind("SUPER + mouse:273", "Resize window", hl.dsp.window.resize(), { mouse = true })

-- Cycling.
bind("ALT + Tab", "Next window", function()
  hl.dispatch(hl.dsp.window.cycle_next())
  hl.dispatch(hl.dsp.window.bring_to_top())
end)
bind("ALT + SHIFT + Tab", "Previous window", function()
  hl.dispatch(hl.dsp.window.cycle_next({ next = false }))
  hl.dispatch(hl.dsp.window.bring_to_top())
end)
bind("CTRL + ALT + Tab", "Next monitor", hl.dsp.focus({ monitor = "+1" }))
bind("CTRL + ALT + SHIFT + Tab", "Previous monitor", hl.dsp.focus({ monitor = "-1" }))

-- Groups (tabbed windows).
bind("SUPER + G", "Toggle group", hl.dsp.group.toggle())
bind("SUPER + ALT + G", "Move window out of group", hl.dsp.window.move({ out_of_group = true }))
bind("SUPER + ALT + Tab", "Next window in group", hl.dsp.group.next())
bind("SUPER + ALT + SHIFT + Tab", "Previous window in group", hl.dsp.group.prev())
bind("SUPER + ALT + mouse_down", "Next window in group", hl.dsp.group.next())
bind("SUPER + ALT + mouse_up", "Previous window in group", hl.dsp.group.prev())
for index = 1, 5 do
  bind("SUPER + ALT + code:" .. (index + 9), "Group window " .. index, hl.dsp.group.active({ index = index }))
end

---------------------------------------------------------------------------
-- Workspaces: each screen has its own 1-10, and the groups of unplugged
-- screens wait on the primary one (workspaces.lua). Next and previous walk the
-- focused screen's groups as its bar shows them and the swipe walks them, the
-- empty workspaces each group keeps included. `previous_per_monitor` stays on
-- the focused screen.
---------------------------------------------------------------------------

for slot = 1, dot.workspaces.slots do
  local key = "code:" .. (slot + 9)
  bind("SUPER + " .. key, "Go to workspace " .. slot, dot.workspaces.focus(slot))
  bind("SUPER + SHIFT + " .. key, "Move window to workspace " .. slot, dot.workspaces.move_window(slot))
  bind("SUPER + SHIFT + ALT + " .. key, "Move window silently to workspace " .. slot,
    dot.workspaces.move_window(slot, false))
end
bind("SUPER + SHIFT + CTRL + Right", "Move window to a new workspace", dot.workspaces.move_window_to_new())

bind("SUPER + Tab", "Next workspace", dot.workspaces.next())
bind("SUPER + SHIFT + Tab", "Previous workspace", dot.workspaces.previous())
bind("SUPER + CTRL + Right", "Next workspace", dot.workspaces.next())
bind("SUPER + CTRL + Left", "Previous workspace", dot.workspaces.previous())
bind("SUPER + CTRL + ALT + Right", "Next workspace group", dot.workspaces.next_group())
bind("SUPER + CTRL + ALT + Left", "Previous workspace group", dot.workspaces.previous_group())
bind("SUPER + CTRL + Tab", "Former workspace", hl.dsp.focus({ workspace = "previous_per_monitor" }))
bind("SUPER + mouse_down", "Next workspace", dot.workspaces.next())
bind("SUPER + mouse_up", "Previous workspace", dot.workspaces.previous())

-- Three fingers swipe through the workspaces with Hyprland's own gesture,
-- which follows the fingers. It walks the screen's workspaces in id order, the
-- order the binds above walk, and reaches empty ones because each group keeps
-- its first and one after its last (workspaces.lua). It never makes one: the
-- kept empty workspace ends each group, so a swipe past the screen's last
-- stays. Fingers moving left bring the next one.
-- https://wiki.hypr.land/configuring/core/binds/gestures/
-- https://wiki.hypr.land/configuring/core/config-options/#gestures
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
hl.config({ gestures = { workspace_swipe_create_new = false } })

-- Scratchpad. code:49 is the key left of 1 (backslash on Italian).
bind("SUPER + S", "Toggle scratchpad", hl.dsp.workspace.toggle_special("scratchpad"))
bind("SUPER + code:49", "Toggle scratchpad", hl.dsp.workspace.toggle_special("scratchpad"))
bind("SUPER + ALT + S", "Move window to scratchpad", hl.dsp.window.move({ workspace = "special:scratchpad", follow = false }))
bind("SUPER + SHIFT + code:49", "Move window to scratchpad", hl.dsp.window.move({ workspace = "special:scratchpad", follow = false }))

---------------------------------------------------------------------------
-- Shell (Quickshell IPC, docs/CONTRACTS.md)
---------------------------------------------------------------------------

local function qs(target, fn)
  return "qs ipc call " .. target .. " " .. fn
end

-- SUPER + Space (the Command menu) is in fallback.lua.
bind("SUPER + Escape", "Power menu", qs("powermenu", "toggle"))
bind("XF86PowerOff", "Power menu", qs("powermenu", "toggle"), locked)
bind("SUPER + CTRL + Escape", "Lock", "loginctl lock-session")
bind("SUPER + CTRL + A", "Control center (audio)", qs("controlcenter", "toggle"))
bind("SUPER + CTRL + B", "Control center (Bluetooth)", qs("controlcenter", "toggle"))
bind("SUPER + CTRL + W", "Control center (Wi-Fi)", qs("controlcenter", "toggle"))
bind("SUPER + CTRL + comma", "Do not disturb", qs("notifications", "toggleDnd"))
bind("SUPER + SHIFT + comma", "Clear notifications", qs("notifications", "clear"))
bind("SUPER + CTRL + ALT + R", "Restart Quickshell", "qs kill; sleep 0.5; qs")

-- Theme / wallpaper (dot-theme is owned by the theme part of the repo).
bind("SUPER + CTRL + Space", "Wallpaper picker", qs("wallpaper", "toggle"))
bind("SUPER + SHIFT + CTRL + Space", "Next wallpaper", "dot-theme next")
bind("SUPER + CTRL + ALT + Space", "Switch wallpaper folder", "dot-theme toggle-folder")

-- Idle on/off (stops hypridle: no auto-lock, no screen-off, no suspend).
bind("SUPER + CTRL + I", "Toggle idle (lock/suspend)",
  "pkill -x hypridle && hyprctl notify -1 3000 'rgb(ffb4ab)' 'Idle off: no auto-lock, no suspend. I stay up all night'"
  .. " || { hyprctl notify -1 3000 'rgb(c2c1ff)' 'Idle on'; exec hypridle; }")

---------------------------------------------------------------------------
-- Look
---------------------------------------------------------------------------

local gaps_on = true
bind("SUPER + SHIFT + BackSpace", "Toggle gaps", function()
  gaps_on = not gaps_on
  hl.config({ general = { gaps_in = gaps_on and 4 or 0, gaps_out = gaps_on and 8 or 0 } })
end)

bind("SUPER + CTRL + Z", "Zoom in", function()
  local zoom = hl.get_config("cursor.zoom_factor") or 1
  hl.config({ cursor = { zoom_factor = math.min(zoom + 1, 10) } }) -- 10 = Hyprland's max
end)
bind("SUPER + CTRL + ALT + Z", "Reset zoom", function()
  hl.config({ cursor = { zoom_factor = 1 } })
end)

---------------------------------------------------------------------------
-- Screenshots and color picker (grim + slurp + wl-clipboard, satty, hyprpicker)
---------------------------------------------------------------------------

bind("Print", "Screenshot region", "dot-hypr-screenshot region")
bind("SUPER + SHIFT + S", "Screenshot region", "dot-hypr-screenshot region")
bind("SHIFT + Print", "Screenshot monitor", "dot-hypr-screenshot output")
bind("ALT + Print", "Screenshot window", "dot-hypr-screenshot window")
bind("SUPER + SHIFT + A", "Screenshot region and annotate (satty)", "dot-hypr-screenshot region --edit")
bind("SUPER + Print", "Color picker", "pkill -x hyprpicker || hyprpicker -a")

---------------------------------------------------------------------------
-- Screen recording (gpu-screen-recorder, scripts/dot-hypr-record). Each
-- record bind starts, or stops whatever recording runs. Add --audio / --mic
-- for sound. Videos go to ~/Videos/Recordings.
---------------------------------------------------------------------------

bind("SUPER + SHIFT + R", "Record region (again to stop)", "dot-hypr-record region")
bind("SUPER + ALT + R", "Record monitor (again to stop)", "dot-hypr-record output")
bind("SUPER + SHIFT + ALT + R", "Record monitor with sound (again to stop)", "dot-hypr-record output --audio")
bind("SUPER + CTRL + R", "Replay buffer on/off (last 30 s)", "dot-hypr-record replay toggle")
bind("SUPER + SHIFT + P", "Save the last 30 s (replay)", "dot-hypr-record replay save")

---------------------------------------------------------------------------
-- Universal copy/paste (Omarchy): SUPER + C/V/X/A like on a Mac. In
-- terminals copy/paste become CTRL + SHIFT + C/V.
---------------------------------------------------------------------------

local function send_once(mods, key)
  -- down/up split works around send_shortcut leaving keys stuck (Omarchy).
  return function()
    hl.dispatch(hl.dsp.send_key_state({ mods = mods, key = key, state = "down" }))
    hl.timer(function()
      hl.dispatch(hl.dsp.send_key_state({ mods = mods, key = key, state = "up" }))
    end, { timeout = 50, type = "oneshot" })
  end
end

local function is_terminal()
  local w = hl.get_active_window()
  if not w then return false end
  for _, t in ipairs(w.tags or {}) do
    if t:gsub("%*$", "") == "terminal" then return true end
  end
  return false
end

local function universal(key)
  local plain, term = send_once("CTRL", key), send_once("CTRL SHIFT", key)
  return function()
    if is_terminal() then term() else plain() end
  end
end

bind("SUPER + C", "Copy", universal("C"))
bind("SUPER + V", "Paste", universal("V"))
bind("SUPER + X", "Cut", send_once("CTRL", "X"))
bind("SUPER + A", "Select all", send_once("CTRL", "A"))

---------------------------------------------------------------------------
-- Keyboard layouts (config/hypr/input.lua takes them from the user's
-- keyboard settings, else /etc/default/keyboard). Windows and GNOME switch on SUPER + Space, which
-- is the Command menu here, so the switch takes SHIFT with it. It is a bind,
-- not an XKB `grp:` option, so the Command menu finds it with the others.
---------------------------------------------------------------------------

bind("SUPER + SHIFT + Space", "Next keyboard layout", "hyprctl switchxkblayout all next")

---------------------------------------------------------------------------
-- Media, volume, brightness (wpctl, brightnessctl, playerctl).
-- Quickshell draws the OSD by watching PipeWire and the backlight.
---------------------------------------------------------------------------

bind("XF86AudioRaiseVolume", "Volume up", "wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+", locked_repeat)
bind("XF86AudioLowerVolume", "Volume down", "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-", locked_repeat)
bind("ALT + XF86AudioRaiseVolume", "Volume up 1%", "wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 1%+", locked_repeat)
bind("ALT + XF86AudioLowerVolume", "Volume down 1%", "wpctl set-volume @DEFAULT_AUDIO_SINK@ 1%-", locked_repeat)
bind("XF86AudioMute", "Mute", "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle", locked)
bind("XF86AudioMicMute", "Mute microphone", "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle", locked)

bind("XF86MonBrightnessUp", "Brightness up", "brightnessctl -e4 -n2 set 5%+", locked_repeat)
bind("XF86MonBrightnessDown", "Brightness down", "brightnessctl -e4 -n2 set 5%-", locked_repeat)
bind("ALT + XF86MonBrightnessUp", "Brightness up 1%", "brightnessctl -n2 set 1%+", locked_repeat)
bind("ALT + XF86MonBrightnessDown", "Brightness down 1%", "brightnessctl -n2 set 1%-", locked_repeat)
bind("SHIFT + XF86MonBrightnessUp", "Brightness max", "brightnessctl set 100%", locked)
bind("SHIFT + XF86MonBrightnessDown", "Brightness min", "brightnessctl -n2 set 1%", locked)
bind("XF86KbdBrightnessUp", "Keyboard light up", "brightnessctl -d '*kbd_backlight*' set 25%+", locked_repeat)
bind("XF86KbdBrightnessDown", "Keyboard light down", "brightnessctl -d '*kbd_backlight*' set 25%-", locked_repeat)

bind("XF86AudioPlay", "Play/pause", "playerctl play-pause", locked)
bind("XF86AudioPause", "Play/pause", "playerctl play-pause", locked)
bind("XF86AudioNext", "Next track", "playerctl next", locked)
bind("XF86AudioPrev", "Previous track", "playerctl previous", locked)
bind("XF86AudioStop", "Stop", "playerctl stop", locked)

---------------------------------------------------------------------------
-- Lid: turn the laptop panel off when closed with an external connected,
-- back on when opened (scripts/dot-hypr-lid). Suspend on lid close is
-- logind's job (battery only, system/ owns the drop-in).
-- Check the switch name with: hyprctl devices  (section "Switches").
---------------------------------------------------------------------------

bind("switch:on:Lid Switch", "Lid closed", "dot-hypr-lid close", locked)
bind("switch:off:Lid Switch", "Lid opened", "dot-hypr-lid open", locked)
