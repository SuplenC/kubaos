# CanvasPage, Canvas and CanvasObject

A template: a region of a screen's layout, empty of any app's content ([DESIGN.md, "Levels and where they live"](../../DESIGN.md#levels-and-where-they-live)). [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every part keeps.

## `CanvasPage`

Rust `template::canvas_page`; iced only, inside a [SidebarPage](sidebar-page.md) page. Its mockup page is the source of truth: Templates, CanvasPage, in `design/mockups/`, live in a window that resizes from the app's minimum to a 1080p screen, with screens to plug in and out.

A canvas of objects drawn to scale, with the selected one's settings: Settings › Displays' screens. It is a region of a SidebarPage page, drawn under the page's header in place of its sub-sections' wells.

**The layout.** On a page under `size.canvas_wide` (1000) the canvas spans the page, `aspect.canvas` (50:21, 42%) of its width tall, capped so the heading of the settings under it stays in view above the page's foot; the settings follow it down the page, which scrolls them. From `size.canvas_wide` the settings go into a column `size.canvas_settings` (400) wide beside the canvas, `space.s32` from it, scrolling on their own (`Scroll`), and the canvas takes all the height left above the page's `space.s24` foot. The canvas fits its objects again on every frame of a resize.

**The settings.** The selected object's settings are ordinary sub-sections of the page: a `Section` with the object's name as its heading, a line of its numbers under it (its size, its mode and scale, where it sits), then wells of rows. They follow the selection, the sidebar's sub-section takes the object's name, and the numbers follow a drag live. The page's other sub-sections come after them. The sidebar's scroll-spy follows the column while the settings are beside the canvas, its lead line the column's top.

**The hardware is live.** A screen plugged in fades in over `medium` on `standard` at its box in the new fit while the others glide to theirs; one unplugged, or the laptop's with its lid closed, fades out where it was, and the selection goes to the first screen left if it was on it. Applying a change and its revert countdown are the app's.

**The keyboard.** The canvas is the page's first stop after its header, then its toolbar's buttons, then the settings' wells. ← → (h, l) pick the object before or after, left to right, round from either end. Shift with an arrow, or H J K L, moves the selected object to its next snap that way: the nearest place where one of its edges or its centre meets a neighbour's; the arrangement fits again. Enter goes into the settings, at the first row of the selected object's.

*Not built yet: `kubaos-ui` has no templates.*

## `Canvas`

Part of `CanvasPage`; it reports itself as `Canvas`, holding content.

A well in `surface_container_lowest` at the content radius, its sunken hairline (the soft outline) and its focus tube drawn over what it holds, and on every `space.s16` of its grid from its corner a dot one device pixel square in the soft outline. Its toolbar is its top band, `space.s12` in: the current desk as a `Chip`, then the canvas's actions at the far end as Text sm buttons (Identify, Mirror). Its `KeyHints` are its bottom band, `space.s12` in. Between them is the stage, where the objects are fitted together, scaled to the stage less `space.s24` all round and centred, every edge rounded to a whole device pixel so two objects that touch share their pixel. With the keyboard on it the tube lights round the well, as a list's does, its glow cut by nothing it holds.

## `CanvasObject`

Part of `CanvasPage`; it reports itself as `CanvasObject`.

An object drawn to scale in logical pixels: `surface_container_high` at the control radius with the soft outline's hairline inside its edge, its number in `headline` and its size in `mono` and `on_surface_variant` under it, centred. One too small for its size (its words and `space.s8` either side across, or the two lines and `space.s8` above and below) shows its number alone.

Under the pointer it lights as a control does; the selected one burns the neon tube inside its edge, with the glow outside at night and as glass by day. A press selects it and starts a drag: it tracks the pointer every frame, nothing easing it, and its left, centre and right, and its top, middle and bottom, snap to any neighbour's within `space.s8` on the canvas, the nearest on each axis winning, with a `primary` hairline across the stage where the snap holds, on and off at once. Letting go applies the change, and the arrangement fits the canvas again, every object gliding to its new box over `medium` on `standard_decel`; reduce motion puts them there at once.
