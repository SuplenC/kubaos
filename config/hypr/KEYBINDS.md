# KubaOS keybinds

You never need the mouse. These are why. `SUPER + F1` opens this file, `q`
closes it. Ported from Omarchy 4 with movement on hjkl.
Number keys are bound by keycode, so `1`..`9` are the top-row keys on the
Italian layout too. Live list with descriptions: `hyprctl binds`.

## Always available (fallback.lua)

| Keys | Action |
|---|---|
| `SUPER + Return` | Terminal (Ghostty; foot if Ghostty is missing) |
| `SUPER + ALT + Return` | foot (fallback terminal) |
| `SUPER + Space` | Launcher (Quickshell; fuzzel if the shell is down) |
| `SUPER + ALT + Space` | fuzzel (fallback launcher) |
| `SUPER + W` | Close window |
| `SUPER + SHIFT + CTRL + R` | Reload Hyprland config |
| `SUPER + SHIFT + CTRL + L` | Lock with hyprlock (no shell needed) |
| `SUPER + SHIFT + CTRL + Escape` | Exit Hyprland |

## Apps

| Keys | Action |
|---|---|
| `SUPER + SHIFT + Return` / `SUPER + SHIFT + B` | Chrome |
| `SUPER + SHIFT + ALT + B` | Chrome incognito |
| `SUPER + SHIFT + F` | File manager (yazi in Ghostty) |
| `SUPER + SHIFT + N` | Neovim |
| `SUPER + CTRL + Return` | Ghostty attached to tmux session `main` |
| `SUPER + SHIFT + D` | lazydocker |
| `SUPER + CTRL + T` | btop |
| `SUPER + F1` | This cheat sheet |

## Windows

| Keys | Action |
|---|---|
| `SUPER + h/j/k/l` (or arrows) | Focus left/down/up/right |
| `SUPER + SHIFT + h/j/k/l` | Move window |
| `SUPER + SHIFT + arrows` | Swap window |
| `SUPER + CTRL + h/j/k/l` | Resize (hold to repeat) |
| `SUPER + '` / `SUPER + ì` (keys right of 0) | Narrower / wider; `+ SHIFT` shorter / taller |
| `SUPER + Q` | Close window |
| `CTRL + ALT + Delete` | Close all windows |
| `SUPER + T` | Toggle floating |
| `SUPER + F` | Fullscreen |
| `SUPER + ALT + F` | Maximize (keeps gaps and bar) |
| `SUPER + O` | Pop out: float + pin + center (again to undo) |
| `SUPER + E` | Toggle split direction (dwindle) |
| `SUPER + P` | Pseudo-tile |
| `SUPER + drag left / right mouse` | Move / resize |
| `ALT + Tab` / `ALT + SHIFT + Tab` | Next / previous window |
| `CTRL + ALT + Tab` / `+ SHIFT` | Next / previous monitor |

## Groups (tabs)

| Keys | Action |
|---|---|
| `SUPER + G` | Toggle group |
| `SUPER + ALT + G` | Move window out of group |
| `SUPER + ALT + arrows` | Move window into group in that direction |
| `SUPER + ALT + Tab` / `+ SHIFT` | Next / previous tab |
| `SUPER + CTRL + Left/Right` | Previous / next tab |
| `SUPER + ALT + 1..5` | Jump to tab n |

## Workspaces

1-3 laptop, 4-6 external 1, 7-9 external 2.

| Keys | Action |
|---|---|
| `SUPER + 1..9` | Go to workspace |
| `SUPER + SHIFT + 1..9` | Move window there (follow) |
| `SUPER + SHIFT + ALT + 1..9` | Move window there (stay) |
| `SUPER + Tab` / `SUPER + SHIFT + Tab` | Next / previous workspace |
| `SUPER + CTRL + Tab` | Last workspace |
| `SUPER + scroll` | Next / previous workspace |
| `SUPER + SHIFT + ALT + h/j/k/l` (or arrows) | Move workspace to monitor |
| `SUPER + S` / `SUPER + \` | Toggle scratchpad |
| `SUPER + ALT + S` / `SUPER + SHIFT + \` | Send window to scratchpad |
| 3-finger swipe | Switch workspace |

## Shell, theme, system

| Keys | Action |
|---|---|
| `SUPER + Escape` / power key | Power menu |
| `SUPER + CTRL + Escape` | Lock (`loginctl lock-session`) |
| `SUPER + CTRL + A` / `B` / `W` | Control center (audio, Bluetooth, Wi-Fi) |
| `SUPER + CTRL + ,` | Do not disturb |
| `SUPER + SHIFT + ,` | Clear notifications |
| `SUPER + CTRL + Space` | Wallpaper picker |
| `SUPER + SHIFT + CTRL + Space` | Next wallpaper (`dot-theme next`) |
| `SUPER + CTRL + ALT + Space` | Switch wallpaper folder (`dot-theme toggle-folder`) |
| `SUPER + CTRL + I` | Idle on/off (no auto-lock or suspend while off) |
| `SUPER + CTRL + ALT + R` | Restart Quickshell |
| `SUPER + SHIFT + BackSpace` | Toggle gaps |
| `SUPER + CTRL + Z` / `+ ALT` | Zoom in / reset zoom |

## Clipboard (macOS style)

| Keys | Action |
|---|---|
| `SUPER + C` / `SUPER + V` | Copy / paste (sends CTRL+SHIFT+C/V in terminals) |
| `SUPER + X` / `SUPER + A` | Cut / select all |

## Screenshots

Saved to `~/Pictures/Screenshots/` and copied to the clipboard. The
notification's "Edit" opens it in satty, which saves `<name>-annotated.png`
next to it and copies it (Enter).

| Keys | Action |
|---|---|
| `Print` / `SUPER + SHIFT + S` | Region |
| `SUPER + SHIFT + A` | Region, then annotate in satty |
| `SHIFT + Print` | Focused monitor |
| `ALT + Print` | Active window |
| `SUPER + Print` | Color picker (hex to clipboard) |

## Screen recording

gpu-screen-recorder, saved to `~/Videos/Recordings/` (mp4, H.264). A red dot
with the elapsed time shows in the bar; click it to stop. The same key stops
any running recording. Sound is off unless the bind says so.

| Keys | Action |
|---|---|
| `SUPER + SHIFT + R` | Record a region / stop |
| `SUPER + ALT + R` | Record the focused monitor / stop |
| `SUPER + SHIFT + ALT + R` | Record the focused monitor with system sound / stop |
| `SUPER + CTRL + R` | Replay buffer on/off (keeps the last 30 s in RAM) |
| `SUPER + SHIFT + P` | Save the last 30 s |

## Media keys

Volume, mute, mic mute, brightness, keyboard backlight, play/pause/next/prev.
`ALT +` volume/brightness steps by 1%. `SHIFT + brightness` jumps to max/min.
All work on the lock screen.
