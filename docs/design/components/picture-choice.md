# PictureChoice

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `PictureChoice`

Rust `picture::picture_choice(picture, aspect, name)`; QML `PictureChoice`.

A picture to pick: the picture covers the whole tile, edge to edge in its own rounded shape at the control radius (12), a `Media` box, with no frame round it, so the picture is as large as the tile. Picked, its edge lights as the focus tube, with its glow (the accent; cold where the keyboard is on a picked one, since the accent already means picked) and its mark shows in a `Selected` chip 8 in from the top right: a check, "Current", as each use names it. The keyboard on it lights the tube too. A badge (a `Neutral` chip, "Video") sits 8 in from the top left whatever its state. The name is `small`, one line, under the picture 8 below it (`name-under`), or over its foot 12 in, light on the scrim darkening from 40% down to the backdrop's 0.90 at the foot (`name-over`), whatever the mode, since it is on a photo. Kōbō's wallpapers (several picked, name over, a check) and the shell's wallpaper picker (one applied, name under, "Current", and "Playing" or "Video") are this one component.

**States:** resting, hovered, pressed, picked (the tube and its mark), focused, picked and focused (the cold tube), loading (the placeholder until the picture arrives), and disabled (`PictureChoice::disabled`, `enabled: false`): the picture at the disabled opacity, its placeholder the disabled container, its name and chips dimmed, no tube, no press, and no stop for Tab.
