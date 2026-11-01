# Media

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Media`

Rust `media::media(look, picture, aspect, corner)`; QML `Media`. Its mockup page is the source of truth: Molecules, Media, in `design/mockups/`.

A picture in a box as wide as its room and as tall as an `[aspect]` token makes it, down to the 4 px grid (so boxes side by side, whose widths can differ by a device pixel, come out one height and their feet line up), cut to cover the box at its corner: the control radius in a picker, nested in a card. Until the picture has loaded the box holds a block `Skeleton` of its own shape, breathing; a picture that can't be read leaves it held still. So the box is the same before, after and without the load, and nothing around it moves (`kubaos-ui` tests it). Dimmed, in a disabled `PictureChoice`, the picture is at `opacity.disabled_content` and the placeholder the disabled container. A caption, where its place gives one, is `small` in `on_surface_variant` `space.s8` under the box. In QML the picture is `source`, or content laid over the placeholder to fill the box (a video, a picture that clips itself), which fades in over it. A `PictureChoice` is built on one.

*Not built yet: neither library draws a caption.*
