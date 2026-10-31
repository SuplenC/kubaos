# Spinner

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Spinner`

Rust `progress::spinner`; QML `Spinner`.

Work that doesn't know: an icon size (16, 20 or 24 beside text; 32 and 48 for a page's one spinner), a lit stretch of neon running round an unlit glass ring. The ring is the tone at `opacity.spinner_track`; the arc, `spinner.arc` of it with round ends, is the tone's core at night with the content glow round it, and the tone by day with no glow. It turns once every `spinner.turn` at one length, never breathing, and keeps turning with reduce motion, since a spinner standing still reads as a hang. The caller passes the time.
