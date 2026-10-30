# Collapsible

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Collapsible`

Rust `collapsible::collapsible`; QML `Collapsible`.

A titled part that opens to show more: a header as tall as a row (44), its title in `body_strong` and a chevron at its end, down while closed and up while open (`open`); open, the content 8 under it and in line with the title, 12 in at the sides, so the content brings no inset of its own. The shell reveals it over `medium` (`standard_decel`) and hides it over `short`. The system monitor's process details.

**States:** closed and open, the header resting, hovered, pressed, focused and disabled (no `on_toggle`, `enabled: false`): its title and chevron at the disabled opacity, no hover or press, and no stop for Tab. What it holds is the owner's to dim with it, and in iced to disable; in QML it takes no input with its header, since Qt's `enabled` passes down.
