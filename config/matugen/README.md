# matugen theme pipeline

`dot-theme` (in `scripts/`) picks a wallpaper and runs matugen 4.2:

```
matugen image <wallpaper> -c ~/.config/matugen/config.toml -m dark \
  -t scheme-fidelity --source-color-index 0 --continue-on-error -q
```

matugen extracts a source color, builds a Material 3 dark scheme, and renders
every `[templates.*]` entry in `config.toml` into `~/.local/state/theme/`.
After that, `dot-theme` reloads tmux, Ghostty (SIGUSR2), Neovim (`:ThemeReload`
on each `$XDG_RUNTIME_DIR/nvim.*` socket), Hyprland
(`hyprctl eval 'dot.reload_colors()'`, falling back to `hyprctl reload`) and
GTK 3 (a quick `gtk-theme` bounce, only if gsettings works). Quickshell watches
`colors.json` on its own, open zsh prompts re-init oh-my-posh when
`ohmyposh.toml` changes, and the other terminal apps read their file at start.

Selection everywhere is `secondary_container` / `on_secondary_container`, a
tinted bar under normal text, never the light ANSI blue.

`dot-theme apply` runs at every login, so it skips matugen and the reloads
when nothing changed since the last successful run: same wallpaper path, mtime
and size, same scheme, same matugen binary, same content of `config.toml`,
`templates/*` and the oh-my-posh template, and every output file present.
That record is `~/.local/state/theme/.stamp`. `dot-theme apply --force`
regenerates anyway; `set`, `next`, `prev` and the folder commands always do.

## Scheme

The scheme type is set in one place, `SCHEME` at the top of `scripts/dot-theme`
(`DOT_THEME_SCHEME=scheme-... dot-theme apply` tries another for one run). It
is `scheme-fidelity`, chosen by rendering all six types on every wallpaper in
`wallpapers/`:

- `tonal-spot` (matugen's default) and `rainbow` wash neon out into pastel
  greys. `expressive` rotates the hue away from the photo (the Tokyo Tower
  scene turned lime).
- `vibrant` is saturated but monochrome.
- `content` and `fidelity` keep the photo's own hue in `primary_container`.
  Fidelity also picks a complementary `tertiary`: cyan against Shinjuku's
  amber, gold against purple hour, amber against Tokyo Tower's blue. That
  complement is what makes it read like neon signage.

Contrast was checked (WCAG 2.1) for every wallpaper: `on_surface` on `surface`
is about 14:1, `on_surface_variant` on `surface_container` about 9.6:1, and the
lowest pair, `on_primary_container` on `primary_container`, is 4.7:1. All pass
AA.

## Defaults

`dot-theme apply` with no wallpaper set uses `DEFAULT_WALLPAPER`
(`ukiyo:kabukicho-gate`, the live scene; matugen reads the scene's poster,
the way it reads a video's, see `docs/UKIYO.md`). Without ukiyo it takes the
first wallpaper of the current folder. With no images at all it renders the
templates from `DEFAULT_SEED` (that wallpaper's source color, also
`fallback_color` in `config.toml`) and copies `default-colors.json` to
`colors.json`. The fallbacks built into the consumers use the same palette:
`Theme.qml` defaults, `config/hypr/colors.lua`, the `@thm_*` defaults in
`config/tmux/tmux.conf`, and the variables in `config/hypr/hyprlock.conf`.
If you change the default wallpaper, regenerate them all from
`dot-theme set <it>`'s `colors.json` (with `"wallpaper": ""`).

## Templates

| Template | Output | Notes |
|---|---|---|
| `colors.json` | `colors.json` | Flat M3 keys + `wallpaper` + `mode`, for Quickshell |
| `tmux.conf` | `tmux.conf` | `@thm_*` options + status/pane/message/mode styles |
| `ghostty` | `ghostty` | ANSI 1-6 from the harmonized `[config.custom_colors]` (Tokyo Night's hues as seeds) |
| `nvim.lua` | `nvim.lua` | `return { primary = "#...", ... }`, plus `red green yellow blue magenta cyan` for syntax |
| `hypr.lua` | `hypr.lua` | `return { primary = "#...", ... }` |
| `~/.config/ohmyposh/config.toml.tmpl` | `ohmyposh.toml` | Owned by the ports side. Go templates escaped as `\{{` |
| `gtk.css` | `gtk.css` | libadwaita / adw-gtk3 `@define-color` overrides |
| `qt6ct.colors` | `qt6ct.colors` | 22 roles x 3 groups |
| `fuzzel.ini`, `foot.ini` | same | `include=` from `config/fuzzel/fuzzel.ini` and `config/foot/foot.ini` |
| `lazygit.yml`, `lazydocker.yml` | same | `gui.theme`; lazygit through `LG_CONFIG_FILE`, lazydocker through a link |
| `yazi.toml`, `btop.theme` | same | Read through relative links in `config/yazi/` and `config/btop/themes/` |
| `delta.gitconfig` | same | `[delta]` colors, included by `config/git/config` |
| `fzf`, `atuin.toml`, `eza.yml`, `glow.json` | same | fzf `--color` options file, atuin theme, eza theme, glamour style |
| `hyprlock.colors` | `hyprlock.colors` | `$key = rrggbb` |

matugen 4.2 stops rendering the rest of the templates when an input file is
missing. The oh-my-posh input therefore goes through a link,
`~/.local/state/theme/.ohmyposh.toml.tmpl`, which `dot-theme` points at the real
file or at `templates/empty` before every run.

## Adding a template

1. Create `templates/<name>` using matugen syntax: `{{colors.primary.default.hex}}`.
   You also get `.hex_stripped` and `.rgba`, plus filters such as
   `| lighten: 6.0` and `| set_alpha: 0.5`. `{{image}}` is the wallpaper path and
   `{{mode}}` is `dark`. Write `\{{` for a literal `{{`.
2. Add it to `config.toml`:
   ```toml
   [templates.name]
   input_path = "./templates/<name>"
   output_path = "~/.local/state/theme/<name>"
   ```
3. If the app needs to be told about the change, add a reload function to
   `scripts/dot-theme` and call it from `reload_all`. Keep it fast and make sure
   it never fails.
4. Run `dot-theme apply` (it notices the new template).
