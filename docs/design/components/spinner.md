# Spinner

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Spinner`

Rust `progress::spinner`; QML `Spinner`.

Work that doesn't know: an icon size (16, 20 or 24 beside text; 32 and 48 for a page's one spinner), a thin arc over a faint full ring. The caller passes the time; the arc turns and breathes with it.
