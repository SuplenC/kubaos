# Icon

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Icon`

Rust `icon::icon(look, glyph, size, color)`; QML `Icon` (`name`, `size`, `color`).

A symbol of KubaOS Symbols ([DESIGN.md, "Iconography"](../../DESIGN.md#iconography)), by the Material Symbols name it replaces (`arrow_forward`), in a square box of one of the icon sizes, 16, 20, 24, 32 or 48. Beside text it takes the size of the text's style (`Type::icon()`, `Tokens.type.<style>.icon`; [DESIGN.md, "Icons by type"](../../DESIGN.md#icons-by-type)), never a size of its own. It is drawn from its SVG with the stroke set for its size and the scale it shows at, in whole device pixels, so no line blurs between two.

Its colour is a role. The colour's alpha goes to the drawing's opacity, so an icon in a disabled control, or on a fading screen, dims with its label. It has no states of its own. The geometry check reads it as an `icon` node with its symbol's name, and holds an icon alone in its slot to the slot's centre (`icon-centre`).
