# desktop/

The desktop: Hyprland, the shell (bar, popups, Command menu look), keyboard, pointer,
clock, system monitor, files, wallpapers and the phone link.

| Folder | Files | What they change | Applies |
|---|---|---|---|
| `hypr/` | `hyprland.lua` | Gaps, rounding, binds, screens, input: any Hyprland setting | `hyprctl reload` |
| | `hypridle.conf` | Extra idle listeners: dim, lock or screens off sooner | restart hypridle |
| | `hyprlock.conf` | The fallback lock screen | its next start |
| `quickshell/` | `settings.json` | The shell's settings: the Command menu's motion | at once |
| `cursor/` | `cursor.toml` | Pointer style and size | at once from the menu; `kubaos cursor apply` after a hand edit |
| `clock/` | `clock.toml`, `holidays.toml` | Extra time zones, holidays, first weekday, countdowns; your holiday rules | the popup's next request |
| `keyboard/` | `keyboard.toml` | Layouts, Mac-style accents on Left Alt | at once from the menu; `kubaos keyboard accents apply` and `hyprctl reload` after a hand edit |
| `sysmon/` | `sysmon.toml` | The bar's monitor pill, its style, metrics, alerts and thresholds | within a few seconds |
| `files/` | `tidy-downloads`, `no-automount` | Flag files, no contents: one turns on the daily Tidy Downloads, the other turns off mounting drives as they're plugged in. Set them from the Command menu | at once |
| `phone/` | `phone-link` | A flag file, no contents: the phone link is on and KDE Connect runs. "Set up a phone" in the Command menu writes it, "Turn the phone link off" removes it | at once from the menu; the next login after a hand edit |
| `wallpapers/` | `sets` | The wallpaper sets the picker shows, one per line. The install writes it | the picker's next look |

The Command menu's "Settings: Desktop" opens this folder in Files.
