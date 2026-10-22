-- Window and layer rules. https://wiki.hypr.land/Configuring/Window-Rules/
-- Find a window's class/title with: hyprctl clients
--
-- Pattern: rules add tags ("+name"), and one rule per tag applies the
-- effects. Tag-effect rules come last so they see every tag.

local function tag(match, name)
  hl.window_rule({ match = match, tag = "+" .. name })
end

---------------------------------------------------------------------------
-- General
---------------------------------------------------------------------------

-- Apps asking to maximize themselves would break tiling (Hyprland example).
hl.window_rule({ name = "suppress-maximize", match = { class = ".*" }, suppress_event = "maximize" })

-- XWayland drag-and-drop ghosts must not steal focus (Hyprland example).
hl.window_rule({
  name = "fix-xwayland-drags",
  match = { class = "^$", title = "^$", xwayland = true, float = true, fullscreen = false, pin = false },
  no_focus = true,
})

---------------------------------------------------------------------------
-- Terminals (the tag is used by the universal copy/paste binds)
---------------------------------------------------------------------------

tag({ class = "^(com\\.mitchellh\\.ghostty|foot|footclient|org\\.codeberg\\.dnkl\\.foot)$" }, "terminal")
-- Both draw a ~85% opaque background (config/ghostty/config, config/foot/foot.ini);
-- decoration:blur (looks.lua) frosts what shows through. Windows with alpha
-- are blurred by default, so no rule is needed; keep them out of no_blur rules.
-- Ghostty scrolls very fast on touchpads under Hyprland (Omarchy uses 0.2).
hl.window_rule({ match = { class = "^com\\.mitchellh\\.ghostty$" }, scroll_touchpad = 0.3 })

-- Small floating TUIs launched by binds with --class=dot.float.
tag({ class = "^dot\\.float$" }, "float-medium")

---------------------------------------------------------------------------
-- Dialogs that should float
---------------------------------------------------------------------------

-- Audio, Bluetooth, network applets.
tag({ class = "^(org\\.pulseaudio\\.pavucontrol|pavucontrol|com\\.saivert\\.pwvucontrol)$" }, "float-medium")
tag({ class = "^(blueman-manager|\\.blueman-manager-wrapped|nm-connection-editor|org\\.gnome\\.Calculator)$" }, "float-medium")

-- File pickers: the GTK portal is all dialogs; others by title.
tag({ class = "^xdg-desktop-portal-gtk$" }, "float-large")
-- Exact titles only: a loose "^Open.*" would also float browser windows
-- whose page title starts with "Open".
tag({ title = "^((Open|Save|Select|Choose) (a )?(File|Files|Folder|Directory|Image)|Save As|File Upload|All Files)$" }, "float-large")

-- Polkit agents and keyring prompts.
tag({ class = "^(hyprpolkitagent|org\\.hyprland\\.hyprpolkitagent|polkit-gnome-authentication-agent-1|org\\.kde\\.polkit-kde-authentication-agent-1|gcr-prompter)$" }, "float-small")

-- Bitwarden desktop: open it, copy a password, close it. A centred window,
-- not half the screen. It starts by itself after first-boot setup closes
-- (amd64 only, see autostart.lua).
hl.window_rule({ match = { class = "^([Bb]itwarden)$" }, float = true, center = true, size = { 1100, 750 } })

-- Generic small dialogs.
tag({ class = "^(zenity|yad|org\\.gnome\\.FileRoller|file-roller)$" }, "float-medium")
tag({ title = "^(.* Properties|File Operation Progress)$" }, "float-small")

---------------------------------------------------------------------------
-- Chrome
---------------------------------------------------------------------------

tag({ class = "^(google-chrome|Google-chrome|chromium|Chromium)$" }, "chromium")
-- Chrome sometimes asks to open new windows floating; keep them tiled.
hl.window_rule({ match = { tag = "chromium" }, tile = true })
-- Picture-in-picture (Chrome and others).
tag({ title = "^(Picture.?in.?[Pp]icture)$" }, "pip")
-- "meet.google.com is sharing your screen" bar: hide it.
hl.window_rule({ match = { title = ".*is sharing (your screen|a window).*" }, workspace = "special:hidden silent" })

---------------------------------------------------------------------------
-- Tag effects (keep last)
---------------------------------------------------------------------------

-- Kōbō, such as first-boot setup (kobo --setup, app id kobo): a centred
-- panel, not half a tiled screen. The newest window has focus.
hl.window_rule({ match = { class = "^kobo$" }, float = true, center = true, size = { "monitor_w*0.8", "monitor_h*0.85" } })
hl.window_rule({ match = { tag = "float-small" }, float = true, center = true })
hl.window_rule({ match = { tag = "float-medium" }, float = true, center = true, size = { "monitor_w*0.45", "monitor_h*0.55" } })
hl.window_rule({ match = { tag = "float-large" }, float = true, center = true, size = { "monitor_w*0.6", "monitor_h*0.7" } })
hl.window_rule({
  match = { tag = "pip" },
  float = true,
  pin = true,
  keep_aspect_ratio = true,
  border_size = 0,
  size = { 640, 360 },
  move = { "monitor_w-window_w-24", "monitor_h-window_h-24" },
})

---------------------------------------------------------------------------
-- Layer rules
---------------------------------------------------------------------------

hl.layer_rule({ match = { namespace = "^launcher$" }, blur = true, animation = "popin 90%" }) -- fuzzel
hl.layer_rule({ match = { namespace = "^selection$" }, animation = "fade" })                -- slurp
hl.layer_rule({ match = { namespace = "^hyprpicker$" }, animation = "fade" })

-- Quickshell (config/quickshell). The shell animates its own surfaces, so
-- Hyprland must not animate them too. The frame (bar, border and panels) is
-- drawn at ~94% opacity: blur what shows through, but not its soft shadow.
hl.layer_rule({ match = { namespace = "^qs-frame$" }, blur = true, ignore_alpha = 0.5, no_anim = true })
hl.layer_rule({ match = { namespace = "^qs-(wallpaper|exclusion)$" }, no_anim = true })
hl.layer_rule({ match = { namespace = "^qs-polkit$" }, animation = "fade" })
-- The Command menu (config/quickshell/modules/menu/Menu.qml), translucent like the frame.
-- It animates its own opening and closing.
hl.layer_rule({ match = { namespace = "^kotodama$" }, blur = true, ignore_alpha = 0.5, no_anim = true })
