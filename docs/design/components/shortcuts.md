# Shortcuts

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Shortcuts` and its button

QML `Shortcuts`. Its mockup page is the source of truth: Molecules, Shortcuts, in `design/mockups/`.

Every key a surface takes, so the surface shows no key hints of its own (the wallpaper picker's bar): a `Popover` on step 2, `size.panel` wide (384), 12 inside, its groups 16 apart. A group's name is `label` in `on_surface_variant`, 4 over its rows; a row is at least a compact row tall (28), what its keys do in `body` at its start and the keys at its end, a `Keycap` each, 4 apart, in one column, at least 16 from the words.

- **The button** is a Text `IconButton` (md) with `info`, at the surface's foot. Its tooltip is the name and its key on one line (`Tooltip`, "A name and its key"): Shortcuts, `?`. The popover opens 8 above it from its start, fading in over `long` (`emphasized_decel`) and out over `medium` (`emphasized_accel`); the button's tooltip stays away while it is open.
- **It opens** on a press of its button or `?` anywhere on the surface, and **closes** on a press of its button again, a press outside it, or Esc. Esc closes it first: the surface under it takes Esc only once it is shut.
- **Reduce motion** fades it over `short`.

*Not built yet: no library has it.*
