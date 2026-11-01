# Tile

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Tile`

Rust `tile::tile`; QML `Tile`. Its mockup page is the source of truth: Molecules, Tile, in `design/mockups/`.

A quick toggle: a 20 symbol (`body_strong`'s), then `space.s12` on a `body_strong` title over a `small` status line, `space.s4` apart, each one line ending in an ellipsis, as tall as a two-line row (56) at the control radius, `space.s12` inside, filling its room. A tile with no status line centres its title alone. The control center's Wi-Fi, Bluetooth and the rest, two to a row `space.s8` apart; an odd one takes the whole row.

**On and off.** Off, it is `surface_container_high`, its title `on_surface` and its symbol and status line `on_surface_variant`. On, it is filled with the accent as a switch that is on: at night the unlit fill, `primary_fill`, its symbol, title and status line `on_primary_fill`. A press, Enter or Space toggles it, the two looks fading over `short` on `standard`. A toggle that takes a moment shows the state it is going to at once and says so in its status line (Turning on…).

**States.** Lit as a switch is. Off and lit, the fill rises to `surface_container_highest` and the tube burns inside its edge with the glow outside at night; pressed, the glow brightens, and by day the tone tints it (`opacity.press_tint`). On and lit, the core (`primary_fill_line`) burns inside the fill's edge, `stroke.thin` wide, with the glow outside, and the content brightens to `on_primary_fill_lit`; pressed, the fill lifts to `primary_fill_pressed`. Signs off, the line is glass (`primary_glass` off, `primary_fill_glass` on) with no glow. Disabled (no action, `enabled: false`): the fill `on_surface` at `opacity.disabled_container`, its content at `opacity.disabled_content`, no press and no stop for Tab.

**The detail arrow.** A tile with more behind it ends in one (`detail`): a circle `size.control_sm` across, `space.s12` from its end, a 16 chevron (`chevron_right`) in the tile's symbol colour, no fill, the words ending `space.s12` before it. It is a small control and stays quiet: lit, its own tube burns round it, and the tile goes back to rest while the pointer is on it; pressed, its tube runs white-hot, and by day the tone tints it. → opens it too; it is no stop for Tab.

*Not built yet: both libraries fill an on tile with `primary` and its content `on_primary` (the status line at `opacity.subdued_text`), light it and its arrow with a state layer, and keep a status line's room when it is empty; the mockup draws the rule.*
