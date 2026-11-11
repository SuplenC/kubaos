# PanelPage

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `PanelPage`

QML `PanelPage`, a template: the shell's alone, as only the shell has panels. Its mockup page is the source of truth, with `Sheet`: Templates, Sheet and PanelPage, in `design/mockups/`. *Not built yet: the system monitor, the notifications, the control center, the clock and the wallpaper picker each lay out their own header, scroll and fade inside their `Sheet`.*

The inside of a `Sheet`: what every shell panel is laid out on. From the top:

- **The header band:** the title in `heading` over its subtitle line in `small`, `on_surface_variant`, each one line, eliding at its end; the panel's icon buttons at the band's end, Text and sm, `space.s8` apart and `space.s8` from the title's room, centred on the two lines. It never scrolls.
- **The subtitle line is always there**, so the header never shifts: not while the panel is open, not when twins swap, not when a count goes to zero (`Header`'s rule, "a subtitle always or never"). It is a pun in the panel's own voice (`docs/BRAND.md`), never a count; each panel has lines of its own, which may rotate. The notifications panel alone says its status there: how many are new, why it is silenced, All read, or Silence is golden with nothing at all.
- **The tabs**, if it has them: a `Tabs` across the container's width, `space.s16` under the header. They don't scroll.
- **The body**, `space.s16` under the header or the tabs: the only part that scrolls, a `Scroll` reaching past its box (so a resting control at its edge keeps its whole glow, and the panel's own edge cuts what passes it), with a `HeaderFade` over the gap above it that shows only once content is behind it.
- **The footer band**, if it has one, `space.s16` under the body: the panel's footer action at its end, a Tonal sm button. A panel's own floating control, as the notifications panel's keys button in its bottom right corner `space.s16` in, floats over the body instead and takes no band. It doesn't scroll.

The panel is `space.s16` inside at its top and its foot, and each band and the body take the container's padding at their sides (16 at the panel's width). It is as tall as its content, up to the screen's room inside the frame, from the bar's foot to the frame's bottom edge; past that only the body gives up height and scrolls, and the bands stay. It is `size.panel` (384) wide, or as wide as its content (the clock, the wallpaper picker).

A body that has nothing shows an `EmptyState` (the notifications panel, one quiet line in the middle of its room, Nothing new.), one still loading its rows as skeletons, and one that broke an `ErrorBlock`; the header keeps its pun and its buttons through all of them.

Keys: Tab and Shift+Tab go round the panel in reading order (the header's buttons, the tabs as one stop, the body, the footer's action) and never leave it while it is open; the bar's pills are never stops. Esc goes back a step in the body first, then closes the panel. Opening lands the keyboard on the panel's first thing to do; closing gives it back to the window that had it.
