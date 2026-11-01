# Pill

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Pill`

QML `Pill`. Its mockup page is the source of truth: Molecules, Pill, in `design/mockups/`. *QML only: the bar is the shell's, and no app has one. It is on the QML specimen's "Shell only" page, which the parity check leaves out.*

A group in the bar: a pill a small control tall (`control_sm`, 32), on `surface_container`, its content a row 8 apart (`spacing`) centred on device pixels, with 12 at each end between the curve and the content (`padding`; 8 for a lone symbol, the bell, the proxy, the updates). A pill of its own tone takes `color` and `ink` together: the recording pill's `error_container` with `on_error_container`, the preview pill's `cold` tint with `cold`.

`interactive` makes it take the pointer, lit by the pointer alone: its tube inside its edge in its tone (the accent, or the recording pill's error) with the glow outside at night; pressed, the tube white-hot and the glow brighter, by day its `ink` tinting it (`opacity.press_tint`). It takes a click (`clicked`), a right click (`rightClicked`) and the wheel (`scrolled`, a step up or down per notch). It is never a stop for Tab and draws no focus tube, so a panel's keyboard focus never wanders into the bar; the bar's own actions are Items in the Command menu too, so none needs the bar's keyboard focus. It takes its own pointer rather than a `Pressable`'s, which only clicks: the bar's pills right-click and scroll too.

On an `interactive` pill, `tooltip` shows in a `Hint` under it while the pointer is on it, moved sideways as far as keeps it whole inside the window, 8 from its edge, and kept there as its words change (the status pill's).

When its content changes width, the pill eases to the new width over `medium` on `standard_decel`, as a list reorders, and cuts its content at its own edge meanwhile, so nothing draws over its neighbours. Content that animates its own width every frame isn't put in a pill, whose ease would trail behind it: the workspaces draw a pill of their own.

It isn't a `Chip` (a fact or a pick inside a surface, 24 tall) or a `Badge` (a count or a dot on a symbol's corner): it is the bar's own group, as tall as the bar's controls.

**States:** rest; an interactive one lit and pressed, its tooltip in a hint while the pointer is on it. It is never focused: no Tab stop in the bar.

*Not built yet: `Pill` lays the state layer of its `ink` over its fill while hovered and pressed.*
