# Your KubaOS settings

This folder, `~/.config/kubaos`, is yours. Updates never write to a file in it. KubaOS's
defaults load first and these files last, so whatever you set here wins. Each file starts
with the settings people change most, commented out with KubaOS's value: uncomment a line
and edit it. Delete a file to get its commented example back at the next update (or run
`dot-config apply`).

This README and the one in each group are KubaOS's, kept current by every update; edit
the other files.

In the Command menu, "Settings folder" opens this folder in Files, and "Settings: Desktop",
"Settings: Terminal", "Settings: Editor", "Settings: Tools" and "Settings: Menu" open a
group.

```
~/.config/kubaos/
  desktop/    hypr/ quickshell/ cursor/ clock/ keyboard/ sysmon/ idle/ files/ wallpapers/ workspaces/
  terminal/   ghostty/ foot/ tmux/ zsh/ fastfetch/
  editor/     nvim/
  tools/      git/ lazygit/ yazi/ bat/ bacon/ fuzzel/
  menu/       kotodama/
  home/       your user flake: your own packages
```

## Want to change something? Go here

| To change | Edit |
|---|---|
| Gaps, borders, rounding, your own key binds | `desktop/hypr/hyprland.lua` |
| Screen layout, resolution, scale | `desktop/hypr/hyprland.lua` (`hl.monitor`) |
| Which screen shows which workspace group, whatever port or place it has; the groups' names | `desktop/workspaces/groups.toml`, or "Pin <screen> to group N", "Unpin <screen>" and "Name group N…" in the Command menu |
| Key repeat, pointer speed, touchpad scrolling | `desktop/hypr/hyprland.lua` |
| When the screen locks, turns off or suspends, on battery and plugged in; what closing the lid does | `desktop/idle/idle.toml`, or "Lock after…", "Screens off after…", "Suspend after…", "When the lid closes…" in the Command menu |
| Dimming the screen, or idle listeners of your own | `desktop/hypr/hypridle.conf` |
| The fallback lock screen | `desktop/hypr/hyprlock.conf` |
| Keyboard layouts, Mac-style accents on Left Alt | `desktop/keyboard/keyboard.toml` |
| Pointer style and size | `desktop/cursor/cursor.toml` |
| Extra time zones, holidays, first day of the week, countdowns | `desktop/clock/clock.toml` |
| Your own holiday rules | `desktop/clock/holidays.toml` |
| The bar's system monitor, its alerts and thresholds | `desktop/sysmon/sysmon.toml` |
| The Command menu's motion (full or fades only) | `desktop/quickshell/settings.json` |
| Which wallpaper sets the picker shows | `desktop/wallpapers/sets` |
| Tidy Downloads daily, automount of drives | `desktop/files/` (flag files `tidy-downloads` and `no-automount`; set them from the Command menu) |
| Terminal font, size, transparency | `terminal/ghostty/config` |
| The fallback terminal (foot) | `terminal/foot/foot.ini` |
| tmux prefix, mouse, status bar | `terminal/tmux/tmux.conf` |
| Shell aliases, variables, PATH | `terminal/zsh/zshrc` |
| The greeting in new Ghostty windows | `dot-greeting off` (writes `terminal/zsh/no-greeting`) |
| What fastfetch shows | `terminal/fastfetch/config.jsonc`, then `dot-config apply` |
| What the greeting shows | `terminal/fastfetch/greeting.jsonc`, then `dot-config apply` |
| Neovim options, keymaps, autocmds | `editor/nvim/options.lua`, `keymaps.lua`, `autocmds.lua` |
| Neovim plugins | `editor/nvim/plugins/*.lua` |
| LazyVim extras | `:LazyExtras` in Neovim (kept in `editor/nvim/lazyvim.json`) |
| Plugin versions | `:Lazy update` in Neovim (kept in `editor/nvim/lazy-lock.json`) |
| git settings and aliases | `tools/git/config` |
| Global git ignore patterns | `tools/git/ignore`, then `dot-config apply` |
| lazygit | `tools/lazygit/config.yml` |
| yazi settings and keys | `tools/yazi/yazi.toml`, `tools/yazi/keymap.toml` |
| bat theme, style, tabs | `tools/bat/config`, then `dot-config apply` |
| bacon | `tools/bacon/prefs.toml` |
| The fallback launcher (fuzzel) | `tools/fuzzel/fuzzel.ini` |
| Command menu actions, hidden Items, project folders, quicklinks | `menu/kotodama/user.toml` |
| Your own packages | `kubaos package add <name>`, or `home/home.nix` (your user flake) |
| Where `kubaos suite new` makes projects (`~/Projects`) | `home/home.nix`: `home.sessionVariables.KUBAOS_PROJECTS` |
| The Bitwarden item and folders gh and project env come from (`gh-token`, `env/<project>`) | `home/home.nix`: `KUBAOS_BW_GH_ITEM`, `KUBAOS_BW_ENV_PREFIX` (docs/SECRETS.md) |

Four files have no include in their app, so `dot-config apply` writes the app's file
from KubaOS's and yours: `tools/bat/config`, `tools/git/ignore`,
`terminal/fastfetch/config.jsonc` and `terminal/fastfetch/greeting.jsonc`. Every update runs
it, and a change of wallpaper rewrites fastfetch's two with the new colours between KubaOS's
and yours; after editing one of them, run `dot-config apply` yourself. Every other file applies the way its app reloads its config;
each file says how.

How each app loads its file: docs/CONTRACTS.md, "Defaults and the user layer"
(https://github.com/SuplenC/kubaos/blob/main/docs/CONTRACTS.md). KubaOS's defaults, for
reading: `~/.local/share/kubaos/defaults`.
