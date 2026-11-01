# PictureChoice

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `PictureChoice`

Rust `picture::picture_choice(picture, aspect, name)`; QML `PictureChoice`. Its mockup page is the source of truth: Molecules, PictureChoice, in `design/mockups/`.

A picture to pick: the picture covers the whole tile, edge to edge in its own rounded shape at the control radius (12), a `Media` box, with no frame round it, so the picture is as large as the tile. Picked, the accent's line burns inside its edge with its glow at night, and its mark shows in a `Selected` chip 8 in from the top right: a check, "Current", as each use names it. A badge (a `Neutral` chip, "Video") sits 8 in from the top left whatever its state. The name is `small`, one line, under the picture 8 below it (`name-under`), in `on_surface_variant` and `on_surface` while picked, or over its foot 12 in, light on the scrim darkening from 40% down to the backdrop's 0.90 at the foot (`name-over`), whatever the mode, since it is on a photo. Kōbō's wallpapers (several picked, name over, a check) and the shell's wallpaper picker (one applied, name under, "Current", and "Playing" or "Video") are this one component, in a grid `space.s16` apart.

**States:** rest, picked (the accent's line and its mark), lit, under the pointer or with the keyboard on it (the cold tube in the picked line's place), pressed (the tube white-hot, by day the cold tone's tint), loading (the placeholder until the picture arrives), and disabled (`PictureChoice::disabled`, `enabled: false`): the picture at the disabled opacity, its placeholder the disabled container, its name and chips dimmed, no line, no press, and no stop for the keys. A set is one stop for Tab; the arrows move without picking, since a pick applies a wallpaper, and Enter or Space picks.

*Not built yet: both libraries light an unpicked picture's keyboard focus in the accent and lay a state layer under the pointer; the shell's picker spaces its thumbnails by `focus.offset` twice.*
