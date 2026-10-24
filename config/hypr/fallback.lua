-- Fallback binds. Loaded first, depend on nothing but apt packages
-- (foot, fuzzel, hyprlock) and Hyprland itself, so you can always get a
-- terminal and a launcher even when Quickshell, Nix or the rest of this
-- config is broken. See docs/CONTRACTS.md "Hyprland conventions".

-- Primary terminal is Ghostty (Nix). If it is not on PATH, fall back to foot.
-- `+new-window` asks Ghostty's background instance for a window (D-Bus
-- starts it if needed, home/terminal.nix); when that fails, a Ghostty of
-- its own. HAZARD: pass $HOME, not `home`: Ghostty 1.3.1's instance takes
-- `home` from +new-window as a folder relative to its own.
hl.bind("SUPER + Return",
  hl.dsp.exec_cmd("command -v ghostty >/dev/null 2>&1 && "
    .. '{ ghostty +new-window --working-directory="$HOME" || exec ghostty; } || exec foot'),
  { description = "Terminal (Ghostty, foot if missing)" })
hl.bind("SUPER + ALT + Return", hl.dsp.exec_cmd("foot"), { description = "Fallback terminal (foot)" })

-- The Command menu is Kotodama, drawn by Quickshell. `kotodama toggle` exits
-- non-zero when no menu answers within two seconds (the shell is down) or
-- when Nix is broken and it isn't on PATH, and then fuzzel opens instead.
hl.bind("SUPER + Space",
  hl.dsp.exec_cmd("kotodama toggle || pkill -x fuzzel || fuzzel"),
  { description = "Command menu (Kotodama, fuzzel if the shell is down)" })
hl.bind("SUPER + ALT + Space", hl.dsp.exec_cmd("pkill -x fuzzel || fuzzel"), { description = "Fallback launcher (fuzzel)" })

-- Survival kit: close, reload, lock without the shell, leave.
hl.bind("SUPER + W", hl.dsp.window.close(), { description = "Close window" })
hl.bind("SUPER + SHIFT + CTRL + R", hl.dsp.exec_cmd("hyprctl reload"), { description = "Reload Hyprland config" })
hl.bind("SUPER + SHIFT + CTRL + L", hl.dsp.exec_cmd("pidof hyprlock || hyprlock"), { description = "Lock with hyprlock (fallback lock)" })
-- Leaving stops graphical-session.target first and waits for it, while
-- Hyprland still runs: the portals and the polkit agent then stop cleanly
-- instead of losing the display and dying with an error. `dot-session leave`
-- does the same for the power menu; this one needs nothing from Nix.
hl.bind("SUPER + SHIFT + CTRL + Escape",
  hl.dsp.exec_cmd("timeout 15 systemctl --user stop graphical-session.target; hyprctl dispatch 'hl.dsp.exit()'"),
  { description = "Exit Hyprland" })
