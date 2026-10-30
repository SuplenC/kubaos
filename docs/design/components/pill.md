# Pill

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Pill`

QML `Pill`; *QML only: the bar is the shell's, and no app has one. It is on the QML specimen's "Shell only" page, which the parity check leaves out.*

A group in the bar: a pill a small control tall (`control_sm`, 32), on `surface_container`, its content a row 8 apart (`spacing`) centred on device pixels, with 12 at each end between the curve and the content (`padding`; 8 for a lone symbol, the bell, the proxy, the updates). A pill of its own tone takes `color` and `ink` together: the recording pill's `error_container` with `on_error_container`, the preview pill's `cold` tint with `cold`.

`interactive` makes it take the pointer: the state layer of its `ink` over its fill while hovered and pressed, as every pressable component's, and a click (`clicked`), a right click (`rightClicked`) and the wheel (`scrolled`, a step up or down per notch). It is never a stop for Tab and draws no focus tube, so a panel's keyboard focus never wanders into the bar; the bar's own actions are Items in the Command menu too, so none needs the bar's keyboard focus. It takes its own pointer rather than a `Pressable`'s, which only clicks: the bar's pills right-click and scroll too.

On an `interactive` pill, `tooltip` shows in a `Hint` under it while the pointer is on it, moved sideways as far as keeps it whole inside the window, 8 from its edge, and kept there as its words change (the status pill's).

When its content changes width, the pill eases to the new width over `medium` on `standard_decel`, as a list reorders, and cuts its content at its own edge meanwhile, so nothing draws over its neighbours. Content that animates its own width every frame isn't put in a pill, whose ease would trail behind it: the workspaces draw a pill of their own.

**States:** resting; an interactive one hovered and pressed, its tooltip in a hint while the pointer is on it. It is never focused: no Tab stop in the bar.
