# Media

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Media`

Rust `media::media(look, picture, aspect, corner)`; QML `Media`.

A picture in a box as wide as its room and as tall as an `[aspect]` token makes it, down to the 4 px grid (so boxes side by side, whose widths can differ by a device pixel, come out one height and their feet line up), cut to cover the box at its corner. Until the picture has loaded the box holds the placeholder, `surface_container_high` (a block `Skeleton`, breathing as it shows in QML), so the box is the same before and after the load and nothing around it moves (`kubaos-ui` tests it). In QML the picture is `source`, or content laid over the placeholder to fill the box (a video, a picture that clips itself), which fades in over it. A `PictureChoice` is built on one.
