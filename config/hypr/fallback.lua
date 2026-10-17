-- Fallback binds. Loaded first, depend on nothing but apt packages
-- (foot, fuzzel, hyprlock) and Hyprland itself, so you can always get a
-- terminal and a launcher even when Quickshell, Nix or the rest of this
-- config is broken. See docs/CONTRACTS.md "Hyprland conventions".

-- Primary terminal is Ghostty (Nix). If it is not on PATH, fall back to foot.
hl.bind("SUPER + Return",
  hl.dsp.exec_cmd("command -v ghostty >/dev/null 2>&1 && exec ghostty || exec foot"),
  { description = "Terminal (Ghostty, foot if missing)" })
hl.bind("SUPER + ALT + Return", hl.dsp.exec_cmd("foot"), { description = "Fallback terminal (foot)" })

-- Primary launcher is Quickshell. `qs ipc call` fails when no shell is
-- running, and then fuzzel opens instead.
hl.bind("SUPER + Space",
  hl.dsp.exec_cmd("qs ipc call launcher toggle || pkill -x fuzzel || fuzzel"),
  { description = "Launcher (Quickshell, fuzzel if the shell is down)" })
hl.bind("SUPER + ALT + Space", hl.dsp.exec_cmd("pkill -x fuzzel || fuzzel"), { description = "Fallback launcher (fuzzel)" })

-- Survival kit: close, reload, lock without the shell, leave.
hl.bind("SUPER + W", hl.dsp.window.close(), { description = "Close window" })
hl.bind("SUPER + SHIFT + CTRL + R", hl.dsp.exec_cmd("hyprctl reload"), { description = "Reload Hyprland config" })
hl.bind("SUPER + SHIFT + CTRL + L", hl.dsp.exec_cmd("pidof hyprlock || hyprlock"), { description = "Lock with hyprlock (fallback lock)" })
hl.bind("SUPER + SHIFT + CTRL + Escape", hl.dsp.exit(), { description = "Exit Hyprland" })
