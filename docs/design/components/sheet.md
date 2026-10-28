# Sheet

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Sheet`

Rust `sheet::sheet`; QML `Sheet`.

A panel growing out of an inner edge of the screen frame: the shell's control center, clock, system monitor, power menu, wallpaper picker, OSD and notification stacks. The sheet and the frame are one surface, `surface` at the frame's glass (0.94): where the sheet meets the edge it curves into it through concave fillets of the surface radius, the frame's own inner corner, and its far corners take the surface radius. It is as large as its content, which keeps its own padding, and sits along the edge where its owner puts it, centred on whole device pixels by default. In the shell it slides out from under the frame as it opens, over `long` on `emphasized_decel`, its content riding on its far edge and fading in a beat after the surface moves; it leaves over `medium` on `emphasized_accel`. The owner draws its surface from its `path`, in one layer with the frame's, so the two never double up; `kubaos-ui` draws one open, hanging from the edge above it.
