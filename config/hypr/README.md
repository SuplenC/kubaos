# config/hypr

KubaOS's Hyprland 0.56+ Lua config, plus hypridle and hyprlock (those two still use
hyprlang `.conf`). These are KubaOS's defaults, linked read-only into
`~/.config/hypr` by home-manager; editing them here takes a `home-manager switch`.
Your own settings go in `~/.config/kubaos/hypr/` (`hyprland.lua`, `hypridle.conf`,
`hyprlock.conf`), which load last.

## Files

| File | What |
|---|---|
| `hyprland.lua` | Entry point, `require`s the modules below in order |
| `fallback.lua` | Binds that never depend on Quickshell or Nix (loaded first) |
| `env.lua` | Environment: PATH, Wayland vars for Qt/GTK/Electron/Chrome, cursor size |
| `monitors.lua` | Every screen at its preferred mode, placed by Hyprland; the laptop panel off while the lid is closed on a dock. Your own layout: below |
| `workspaces.lua` | Per-screen workspaces: each screen has its own 1-10 (a block of ten ids, the laptop's first), joins another on unplug and returns on replug. `workspace_blocks.lua` is its bookkeeping, tested by `test/lua/workspaces_test.lua` |
| `input.lua` | Keyboard layout from `/etc/default/keyboard` (`us` without it), repeat rate, touchpad, gestures, the `kubaos:mac-accents` XKB option |
| `accents.lua` | The Mac-style accents' cheat sheet for the Shortcuts View, while they are on |
| `looks.lua` | Gaps, borders, rounding, blur, shadows, animations, dwindle, colors. Sizes and curves match the Quickshell tokens (`config/quickshell/theme/Theme.qml`); `rounding = 10` plus the 2 px border makes the window's outer edge concentric with the frame's 20 px corners at `gaps_out = 8` |
| `colors.lua` | Reads `~/.local/state/theme/hypr.lua`, falls back to built-in colors |
| `rules.lua` | Window rules (floating dialogs, PiP, Chrome, terminals) and layer rules, including Quickshell's (`qs-frame` blurred with `ignore_alpha`, no Hyprland animation on the shell's own layers) |
| `keybinds.lua` | Everything else, ported from Omarchy 4. `SUPER + F1` lists them all in the Command menu |
| `autostart.lua` | Session startup; the polkit agent toggle is at the top |
| `hypridle.conf` | Lock 5 min, screens off 10 min, suspend 30 min on battery only |
| `hyprlock.conf` | Fallback lock screen, themed, fingerprint enabled |
| `~/.config/kubaos/hypr/hyprland.lua` | Yours, not in the repo: loaded last (`docs/CONTRACTS.md`, "Defaults and the user layer") |

Scripts (in `scripts/`, linked to `~/.local/bin`): `dot-hypr-workspaces`
(event-socket listener, runs `dot-hypr-lid sync` on hotplug), `dot-hypr-lid`
(laptop panel off when the lid closes with an external connected),
`dot-hypr-on-ac` (used by hypridle), `dot-hypr-screenshot`.

## Reload

- Hyprland reloads by itself when a config file changes. Manually:
  `hyprctl reload` or `SUPER + SHIFT + CTRL + R`.
- Errors show as a banner and in `hyprctl configerrors`. Check without
  applying: `Hyprland --verify-config`.
- Colors only: `hyprctl eval 'dot.reload_colors()'` (what `dot-theme` should
  call after writing `hypr.lua`; `hyprctl reload` also works).
- hypridle: `pkill hypridle; hypridle &` (or toggle twice with
  `SUPER + CTRL + I`). hyprlock reads its config on every start.
- Poke at the live Lua state: `hyprctl repl`.

## Your screens

The defaults place every screen at its preferred mode. Set your own layout in
`~/.config/kubaos/hypr/hyprland.lua`, one line per screen:

1. Run `hyprctl monitors all`: each screen's `description:` (make, model,
   serial) and `availableModes`.
2. Add a line per screen, matched by description so it follows the screen
   to any port or dock:

   ```lua
   hl.monitor({ output = "desc:LG Electronics LG ULTRAGEAR 312NTXX1A234", mode = "2560x1440@143.99", position = "1920x0", scale = 1 })
   ```

   `position` is the top-left corner in logical pixels (after scale): a
   screen to the right of a 1920-wide one starts at `x = 1920`.
3. `hyprctl reload`.

A line of your own for `eDP-1` replaces the lid rule above it: copy the flag
check from `monitors.lua` around it, or the panel stays on with the lid
closed. Workspaces don't depend on any of this: each screen gets its block
the first time it is seen, by its description, and keeps it
(`~/.local/state/kubaos/hypr/workspace-screens`).

## Workspaces

Each screen has workspaces 1-10 of its own: `SUPER + 1..0` go to the
focused screen's, `SUPER + H/J/K/L` focus another screen. Hyprland numbers
workspaces globally, so `workspaces.lua` gives each screen a block of ten ids
(the laptop panel 1-10, the next screen 11-20) and the bar labels them 1-10.

A screen's workspaces are its group. Unplugging a screen moves its group to
the primary screen (the laptop panel; with the lid closed, the external
Hyprland numbered first) under its own numbers, if it had any windows; an
empty group vanishes. Plugging the screen back returns the group to it. The
bar shows each group on its own, a thin rule between them, e.g. `1 2 3 | 1 2`,
in block order: the screen's own first, then the waiting groups, each on a
tint so it reads as borrowed. Block order is id order, the order Hyprland's
swipe walks a screen's workspaces in, so the one screen that sees a waiting
group before its own is an external holding the laptop's group (1-10) while
the lid is closed.

Each group keeps two workspaces even when they are empty, as Hyprland
persistent workspace rules bound to its screen (`hl.workspace_rule`, set and
released at runtime by `workspaces.lua`): its first, and one after its last
workspace with windows. The bar draws that last one as a faint `+` until it is
shown. Once a window lands on it, the next number is kept in its place and the
one no longer needed is released. A group full up to 10 keeps no empty one
after it.

Next and previous (`SUPER + Tab`, `SUPER + CTRL + →/←`, `SUPER + wheel`, the
bar's wheel) and the three-finger swipe walk the focused screen's workspaces in
that order, the kept empty ones too. The swipe is Hyprland's own, so it follows
the fingers: swipe slowly to peek at the workspace beside. From a group's last
with windows, next opens the empty one after it, and next again goes to the
first of the group on the right; from a group's first, previous goes to the
last of the group on the left. At either end they stay; the swipe never makes a
workspace (`workspace_swipe_create_new` is off, the kept empty workspace takes
its place). `SUPER + CTRL + ALT + →/←` jump to the first workspace of the next
or previous group. `SUPER + 1..0` address the focused group: the screen's own,
or the waiting group whose workspace it shows, so on a waiting group
`SUPER + 2` is that group's 2.

No workspace is renumbered when a screen comes or goes. The Quickshell the
bar runs (v0.3.2) keeps the id a workspace had when it first saw it, so a
renumbered one shows under its old number and a click on it goes nowhere.

Every action is also an expression for the socket:
`hyprctl dispatch 'dot.workspaces.focus(3)'`,
`dot.workspaces.move_window(3)` (`, false` not to follow),
`dot.workspaces.move_window_to_new()`, `dot.workspaces.move_to_monitor("l")`,
`dot.workspaces.next()`, `previous()`, `next_group()`, `previous_group()`
(each takes a connector, the focused screen without one). The bar's clicks go
through `dot.workspaces.go_to(id, "<connector>")`, its wheel through
`dot.workspaces.next("<connector>")` and `previous`.

## Lid

`switch:on/off:Lid Switch` binds call `dot-hypr-lid`. It turns `eDP-1` off
only when another monitor is active, via a flag file in
`$XDG_RUNTIME_DIR/dot-hypr/` that `monitors.lua` reads, then `hyprctl reload`.
If the switch has another name on this machine, check `hyprctl devices` and
fix the two binds at the bottom of `keybinds.lua`. Suspend on lid close is
logind's job (battery only), configured in `system/`.

## Editor support

`.luarc.json` points LuaLS at `/usr/share/hypr/stubs`, which the Hyprland
package ships, for completion of the `hl.*` API.

The option and rule names used in `looks.lua` and `rules.lua` were checked
against the Hyprland v0.56.2 source: `src/config/values/ConfigValues.cpp` for
`decoration:blur:*`/`shadow:*`, and the layer rule effects in
`src/config/lua/bindings/LuaBindingsConfigRules.cpp` (`blur`, `ignore_alpha`,
`no_anim`, `animation`). The Lua syntax is checked with `luac5.5 -p *.lua`.
