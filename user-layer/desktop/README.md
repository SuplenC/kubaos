# desktop/

The desktop: Hyprland, the shell (bar, popups, Command menu look), keyboard, pointer,
clock, system monitor, idle, the power stops, files, wallpapers, the phone link, the work network (Proxy) and the workspace groups.

| Folder | Files | What they change | Applies |
|---|---|---|---|
| `hypr/` | `hyprland.lua` | Gaps, rounding, binds, screens, input: any Hyprland setting | `hyprctl reload` |
| | `hypridle.conf` | Idle listeners of your own, such as dimming the screen | `kubaos-idle apply`, or the next login |
| | `hyprlock.conf` | The fallback lock screen | its next start |
| `quickshell/` | `settings.json` | The shell's settings: the Command menu's motion, the focus stutter | at once |
| `cursor/` | `cursor.toml` | Pointer style and size | at once from the menu; `kubaos cursor apply` after a hand edit |
| `clock/` | `clock.toml`, `holidays.toml` | Extra time zones, holidays, first weekday, countdowns; your holiday rules | the popup's next request |
| `keyboard/` | `keyboard.toml` | Layouts, Mac-style accents on Left Alt | at once from the menu; `kubaos keyboard accents apply` and `hyprctl reload` after a hand edit |
| `sysmon/` | `sysmon.toml` | The bar's monitor pill, its style, metrics, alerts and thresholds | within a few seconds |
| `idle/` | `idle.toml` | After how long idle the screen locks, the screens go off and it suspends, on battery and plugged in, and what closing the lid does. The Command menu's "Lock after…", "Screens off after…", "Suspend after…" and "When the lid closes…" write it | at once from the menu; `kubaos-idle apply` after a hand edit |
| `power/` | `power.toml` | The power stops: what the laptop panel does in each, Super Saver's clock cap, USB devices kept awake, the charge limit and the low-battery drop. The Command menu's power Items write it | at once from the menu; `kubaos power apply` after a hand edit |
| `files/` | `tidy-downloads`, `no-automount` | Flag files, no contents: one turns on the daily Tidy Downloads, the other turns off mounting drives as they're plugged in. Set them from the Command menu | at once |
| `phone/` | `phone-link` | A flag file, no contents: the phone link is on and KDE Connect runs. "Set up a phone" in the Command menu writes it, "Turn the phone link off" removes it | at once from the menu; the next login after a hand edit |
| | `plugins` | KDE Connect plugins a paired phone may use beyond the phone link's own, one name a line (`kdeconnect_mousepad`); every other one stays off | the next start of the phone link |
| `network/` | `proxy.toml`, `vpn.conf` or `vpn.ovpn` | The Proxy, the work network's switch: IT's VPN, WireGuard or OpenVPN (its config without the secrets, which are in the vault), the work DNS, the HTTP proxy or PAC file, and the company's root certificate. "Import VPN config…" in the Command menu writes `vpn.conf` or `vpn.ovpn` and names it, and its type, here | the next "Proxy on" |
| `wallpapers/` | `sets` | The wallpaper sets the picker shows, one per line. The install writes it | the picker's next look |
| `workspaces/` | `groups.toml` | Which screens are pinned to which workspace group, by description, and the groups' names: a pinned screen shows its group wherever it is plugged in. Two of a group's screens plugged in: the first listed wins. Screens without a pin take the groups left over, left to right, never one a group pins. The laptop panel can be pinned too (with the lid closed it holds no group). The Command menu's "Pin <screen> to group N", "Unpin <screen>" and "Name group N…" write it; a mistake is a notification, and no screen is pinned until it is fixed | at once, as it is saved |

The Command menu's "Settings: Desktop" opens this folder in Files.
