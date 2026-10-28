# Badge

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Badge`

Rust `avatar::badge`; QML `Badge`.

What is new about something, over its icon's top right corner: a count in a pill 16 tall (`badge`), `label` type, a circle for one digit and 4 either side for more, 99+ past 99; or a dot, 8 (`badge_dot`). In `error`, with `on_error` on it, so it reads over any icon. Unread notifications on the bar's bell, waiting updates, a proxy that failed. In the bar a badged symbol takes the badge's overhang as room on both sides, so the badge keeps the gap to its neighbours and the symbol stays centred.
