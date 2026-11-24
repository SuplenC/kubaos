# CameraPreview

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

Its mockup page is the source of truth: Molecules, CameraPreview, in `design/mockups/` (`src/lib/camera-preview.ts`).

## `CameraPreview`

A camera's live picture, shown beside the controls that set it (Settings › Camera, in a [`Beside`](beside.md)), so a change is seen as it is made. The picture is a [`Media`](media.md) as wide as its room and `aspect.camera` (16:9) of it tall, down to the grid, at the content radius (16), cut to cover the box; until it arrives the box holds the `Media` skeleton, so nothing around it moves.

**In use by another app.** While another app has the camera the page can't read its picture, and the box says who, in the same box: the list well's sunken fill and soft hairline at the content radius, and centred in it a 32 `video` symbol in `on_surface_variant` over one `subheading` line, `space.s12` apart: "In use by Firefox", or "In use by another app" when the system can't tell which. The line keeps `space.s16` clear of the box's sides and ends in an ellipsis where a long name doesn't fit. The controls beside it still apply.

It takes no pointer and no keyboard: Tab passes it.

*Not built yet: no library has it; it is a mockup only, and in the mockups the picture is a drawn stand-in, never a camera.*
