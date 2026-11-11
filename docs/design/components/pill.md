# Pill

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Pill`

QML `Pill`. Its mockup page is the source of truth: Molecules, Pill, in `design/mockups/`. *QML only: the bar is the shell's, and no app has one. It is on the QML specimen's "Shell only" page, which the parity check leaves out.*

A group in the bar: a pill a small control tall (`control_sm`, 32), on `surface_container`, its content a row 8 apart (`spacing`) centred on device pixels, with 12 at each end between the curve and the content (`padding`; 8 for a lone symbol, the bell, the proxy, the updates). A pill of its own tone takes `color` and `ink` together: the recording pill's `error_container` with `on_error_container`, the preview pill's `cold` tint with `cold`.

A symbol and its own value (a system monitor reading, the battery's share, the recording's time, a scheduled action's time left) are one thing in that row: 4 apart (`space.s4`), the value straight after its symbol, so each pair reads as one beside the 8 or 12 between a pill's things. Where a value is held to a width so its neighbours stay put as it changes (a reading as wide as "100%"), the room it doesn't use falls after it, never between it and its symbol.

`interactive` makes it take the pointer, lit by the pointer alone: its tube inside its edge in its tone (the accent, or the recording pill's error) with the glow outside at night; pressed, the tube white-hot and the glow brighter, by day its `ink` tinting it (`opacity.press_tint`). It takes a click (`clicked`), a right click (`rightClicked`) and the wheel (`scrolled`, a step up or down per notch). It is never a stop for Tab and draws no focus tube, so a panel's keyboard focus never wanders into the bar; the bar's own actions are Items in the Command menu too, so none needs the bar's keyboard focus. It takes its own pointer rather than a `Pressable`'s, which only clicks: the bar's pills right-click and scroll too.

On an `interactive` pill, `tooltip` shows in a `Hint` under it while the pointer is on it, moved sideways as far as keeps it whole inside the window, 8 from its edge, and kept there as its words change (the status pill's).

When its content changes width, the pill eases to the new width over `medium` on `standard_decel`, as a list reorders, and cuts its content at its own edge meanwhile, so nothing draws over its neighbours. Content that animates its own width every frame isn't put in a pill, whose ease would trail behind it: the workspaces draw a pill of their own.

It isn't a `Chip` (a fact or a pick inside a surface, 24 tall) or a `Badge` (a count or a dot on a symbol's corner): it is the bar's own group, as tall as the bar's controls.

**States:** rest; an interactive one lit and pressed, its tooltip in a hint while the pointer is on it. It is never focused: no Tab stop in the bar.

*Not built yet: `Pill` lays the state layer of its `ink` over its fill while hovered and pressed. The system monitor's reading sits at the right of its held box (`SysmonPill.qml`), so its room falls between it and its symbol, and the recording's dot and time are 8 apart (`RecordingIndicator.qml`).*
