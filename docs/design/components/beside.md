# Beside

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

Its mockup page is the source of truth: Layout, Beside, in `design/mockups/` (`src/lib/beside.ts`).

## `Beside`

A column with a picture beside it, at its end, `space.s24` apart: Settings › Camera's controls and its [`CameraPreview`](camera-preview.md). The picture is 40% of the room, held between `size.beside_picture_least` (280) and `size.beside_picture_most` (640), and takes all the column would have past `size.beside_controls_most` (720), so on a wide page the controls stop growing and the picture grows instead; the column takes the rest. Both start at the column's top, and as the column scrolls the picture stays in view at the lead line, `space.s32` below the page's top, the line a page's headings land on.

**Too narrow for both.** Where the column would be narrower than `size.beside_controls_least` (300), in a Settings window under about 916 wide, the picture stands above the column instead, `size.beside_picture_least` wide at its start, `space.s32` over it, and scrolls away with the page.

Each width is worked out from the room and rounded to whole device pixels; crossing the switch point moves the picture at once, as layout never animates. A page holding one takes its pane's whole width (SidebarPage's `wide` section), since the column holds itself to its most.

It takes no keyboard of its own: Tab goes through the column, and the picture has no stop.

*Not built yet: no library has it; it is a mockup only.*
