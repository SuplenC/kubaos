# Collapsible

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Collapsible`

Rust `collapsible::collapsible`; QML `Collapsible`. Its mockup page is the source of truth: Molecules, Collapsible, in `design/mockups/`.

A titled part that opens to show more: a header as tall as a row (44, `size.row`) at the control radius, `space.s12` inside, its title in `body_strong` (`on_surface`, one line, an ellipsis where it doesn't fit) and a chevron (`expand_more`, 20) at its end in `on_surface_variant`; open, the content `space.s8` under it and `space.s12` in at the sides, in line with the title, so the content brings no inset of its own. The header has no fill. The system monitor's process details.

**Opening.** The room under the header eases from nothing to the content's height over `medium` on `standard_decel` and back over `short` on `standard_accel`, cutting the content while it moves and nothing once it is open, so a lit control inside keeps its glow. The chevron turns from down to up over the same time and curve. The header never moves. With reduce motion the room opens and shuts at once, and the content fades in over `short`.

**States.** The header follows the neon as a Text button does: lit (under the pointer or with the keyboard on it), the tube inside its edge, `primary`, with the glow outside at night; pressed, the tube runs white-hot and the glow brightens, and by day the tone tints the header at `opacity.press_tint`; signs off, the tube is `primary_glass` with no glow. Disabled (no `on_toggle`, `enabled: false`): its title and chevron at `opacity.disabled_content`, no press, and no stop for Tab. What it holds is the owner's to dim with it, and in iced to disable; in QML it takes no input with its header, since Qt's `enabled` passes down.

**Keys.** Enter or Space on the header toggles it, once per press; Tab reaches the content's controls only while it is open. The keyboard stays on the header.

*Not built yet: both libraries swap the chevron's glyph instead of turning it, light the header with a state layer, and leave the reveal to the shell (`kubaos-ui` draws it open or shut); the mockup draws the rule.*
