# Switch

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Switch`

Rust `switch::switch`; QML `Switch`.

On or off, 44 × 24, the knob 12 off and 16 on, a hairline track. It takes effect at once; Space, Enter or a click toggles.

**States:** off and on, each resting, hovered, focused and disabled (no `on_toggle`, `enabled: false`).
