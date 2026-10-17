# config/hypr

KubaOS's Hyprland 0.56+ Lua config, plus hypridle and hyprlock (those two still use
hyprlang `.conf`). Live-linked to `~/.config/hypr`, so edits apply without a
home-manager rebuild.

## Files

| File | What |
|---|---|
| `hyprland.lua` | Entry point, `require`s the modules below in order |
| `fallback.lua` | Binds that never depend on Quickshell or Nix (loaded first) |
| `env.lua` | Environment: PATH, Wayland vars for Qt/GTK/Electron/Chrome, cursor size |
| `monitors.lua` | eDP-1 and the two LG UltraGears (fill in their descriptions, below) |
| `workspaces.lua` | 1-3 laptop, 4-6 external 1, 7-9 external 2, persistent |
| `input.lua` | Keyboard layout from `/etc/default/keyboard` (`us` without it), repeat rate, touchpad, gestures |
| `looks.lua` | Gaps, borders, rounding, blur, shadows, animations, dwindle, colors. Sizes and curves match the Quickshell tokens (`config/quickshell/theme/Theme.qml`); `rounding = 10` plus the 2 px border makes the window's outer edge concentric with the frame's 20 px corners at `gaps_out = 8` |
| `colors.lua` | Reads `~/.local/state/theme/hypr.lua`, falls back to built-in colors |
| `rules.lua` | Window rules (floating dialogs, PiP, Chrome, terminals) and layer rules, including Quickshell's (`qs-frame` blurred with `ignore_alpha`, no Hyprland animation on the shell's own layers) |
| `keybinds.lua` | Everything else, ported from Omarchy 4. Cheat sheet: `KEYBINDS.md` |
| `autostart.lua` | Session startup; the polkit agent toggle is at the top |
| `hypridle.conf` | Lock 5 min, screens off 10 min, suspend 30 min on battery only |
| `hyprlock.conf` | Fallback lock screen, themed, fingerprint enabled |
| `local.lua` | Optional, git-ignored machine overrides, loaded last if present |

Scripts (in `scripts/`, linked to `~/.local/bin`): `dot-hypr-workspaces`
(event-socket listener, puts workspaces back on replug), `dot-hypr-lid`
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

## Filling in the monitor descriptions

1. Plug in the dock and run `hyprctl monitors all`.
2. Copy each UltraGear's `description:` line (they differ only by serial).
3. In `monitors.lua`, replace the two `REPLACE-WITH-SERIAL` placeholders:
   `external_1` gets workspaces 4-6, `external_2` gets 7-9.
4. Adjust the `position` values to match the desk, save. Done.

The laptop panel is `eDP-1` at scale 1.5 (1920x1200 logical), the same
density as the 1080p externals at scale 1; the reasoning is in
`monitors.lua`.

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
