# Badge and BadgedIcon

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Badge`

Rust `avatar::badge`, `avatar::accent_badge`; QML `Badge`, `accent` for the second.

What is new about something, over its icon's top right corner: a count in a pill 16 tall (`badge`), `label` type, a circle for one digit and 4 either side for more, 99+ past 99; or a dot, 8 (`badge_dot`). In `error`, with `on_error` on it, so it reads over any icon: waiting updates, a proxy that failed.

`accent` draws it in `primary`, with `on_primary` on it, where it marks something unread rather than something wrong: an unread notification's dot on its app's tile. Red there would make every new message look like a failure.

## `BadgedIcon`

QML `BadgedIcon`; *QML only: the bar's. Its badge hangs past the top of its box, which iced lays out only inside it, so `kubaos-ui` gets it as a widget of its own with the first app that badges a symbol. Until then it is in neither specimen, so the parity check leaves it out.*

An `Icon` (`name`, `size`, `color`; 16 in `on_surface_variant` by default) with a `Badge` centred on its top right corner while `badged`: a count, or a dot when `count` is 0. Its box is the symbol's height; while the badge shows, it takes the half of the badge past the symbol as room on both sides, so the badge keeps the row's gap to the symbol beside it and the symbol stays centred in a pill of its own. The bar's updates (how many security updates wait, or a dot while only a release or an upgrade does) and proxy (a dot while something is wrong with it).
