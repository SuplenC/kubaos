# Wallpaper picker

The shell's wallpaper picker: QML `WallpaperPicker` in `config/quickshell/modules/wallpaper/`, opened by Super+Ctrl+Space, the control center's Wallpaper and the Command menu. Its mockup page is the source of truth: Shell surfaces, Wallpaper picker, in `design/mockups/`. It is built from the library: the `Lock` template, `DepthPicture`, `Tabs`, `Button`, `ButtonBar`, `IconButton`, `Chip`, `Switch`, `List`, `EmptyState`, `Shortcuts` and `Tooltip`.

*Not built yet: the shell's picker is still a bottom-edge sheet of thumbnails with a settings tab; the gallery below is the mockup's.*

## The wall

A gallery. Every picture of a folder hangs on one wall, a floor per folder (KubaOS's own, Mine, Downloads), the wall gliding a picture and its gap a step on `spring` over `long`, the floors over `long` on `emphasized_decel`. The geometry is shares of the screen (`[scale]`), so the wall keeps its shape on every screen:

| Part | Token | |
|---|---|---|
| A picture's height | `scale.gallery_picture` | 0.62 of the screen's, 16:10 |
| The pictures' top edge | `scale.gallery_top` | 0.12 of the screen's height |
| Between two pictures | `scale.gallery_gap` | 0.14 of the screen's width |
| The lamp | `scale.gallery_lamp` | 0.5 of its picture's width, `space.s4` thick, `space.s32` over it |
| The light's reach | `scale.gallery_light_width`, `scale.gallery_light_height` | 0.6 of the screen's width across, 1.0 of its height down |
| Where the light has fallen to a quarter | `scale.gallery_light_falloff` | 0.35 of its reach |

- **The lamp is a neon tube** over each picture, in that picture's own primary, lit or unlit, so a lamp never changes colour: unlit its deep unlit tone, lit its core with the glow at night; by day an outline, and its tone while lit.
- **The room's light is the lamp's.** One field over the whole screen, wall and floor alike, so it has no seam: a radial light from the lit lamp's middle, reaching `scale.gallery_light_width` of the screen across and `scale.gallery_light_height` down, fallen to a quarter at `scale.gallery_light_falloff` of its reach. Lit, it is the lamp's colour at `opacity.lamp_light`; with every lamp off, a grey at `opacity.lamp_dark`. One number, how lit the room is, drives the lamp, its light and the room's colours together, so they never part. By day nothing glows: the room is its surface and only the lamp shows its tone.
- **Browsing is lights off, walk, lights on.** The lit lamp drops off over `short` and the room goes dark and neutral as the wall glides; once it lands, the new picture's lamp strikes over `strike`, the sign's flicker with Flicker on and its steps as fades with Steady, and the room lights in its colour. A step taken before the strike puts it off again, so the room stays dark while you move and only the lamp you stop at strikes. The light only ever goes between the dark grey and one lamp's colour, its chroma rising at that colour's hue, so no two pictures' hues mix. The same holds changing floors. Reduce motion fades the light down and up over `short`.
- **The words and the bar follow the light:** every role of the room keeps its picture's hue, its chroma rising from none and its lightness from the tokens' own palette's as the room lights, so with the lamps off the bar, the name, the depth chip and the floors are the same neutral on every picture, and a new picture's colours come in as its lamp strikes.
- **The one in front of you** is at full size with the ladder's top shadow; the others stand back to `scale.behind` at `opacity.subdued`. Under it the placard: the folder and which of how many in `label` after the 壁紙 accent, the name in `title`, its chips (`Current`, a video's, `Depth` in the cold tone or `Depth off`), and at its end the depth `Switch` with a line on why it says what it says. A picture with no cut-out has its switch disabled: "No cut-out: this picture is flat."
- **The floors** are a `List` at the wall's start, the folder you are on current. An empty folder hangs one frame holding an `EmptyState` that names its folder's path.
- **The colours follow the picture in front of you**: the room recolours to its scheme through grey over `recolour`, and each picture is drawn in its own.

## Each picture is what it would be

Every picture on the wall is the `Lock` template at the screen's size, scaled into its frame, so nothing is laid over a frame and every frame is exact:

- **Picture:** the picture alone, the one in front pushed in with its city off the sky (`DepthPicture`).
- **Desktop:** your desktop on it, the frame, the bar and your windows, in its own scheme in the page's mode.
- **Lock:** your lock on it, the rest clock between the plate and the city, always in its night scheme.

Switching view turns every frame. The one in front plays the whole move: from the desktop, the Lock's own locking (the windows step back, the frame and the bar slide out, the bar's time grows into the rest clock behind the city as the picture pushes in); back, the Lock's own unlocking, the clock flying into the bar's time. Its neighbours play a simpler move, their clocks fading where they stand. Every move eases a number on the element that stays, so its last frame is the rest it leaves. **W** wakes the lock in front or lets it rest, with the Lock's own waking.

**Browsing** glides the wall: each picture's city stands `space.s24` ahead of its sky the farther it is from the front, and catches up on `spring` as it lands, so the city travels faster than the sky; the one reached pushes in, the one left eases back. **D** settles the city onto the sky or lifts it off over `long`; on a lock the clock fades out round the change and back.

## Stepping out

The one in front steps out to fill the screen (F, Space, the bar's Full screen, or a click on it; on the full screen a click on the picture steps back, as F does, while the bar's controls and an awake lock's password sheet keep their own clicks): the room grows round its frame over `slow` until it is the screen, its corner and shadow going, and back over `long`. The live desktop is that picture stepped out, so:

- **Opening** steps the desktop back into its frame as the wall builds round it in a dark room, its frame and bar sliding out as on the wall's picture; the lamp strikes as it lands.
- **Enter** steps the picture in front of you out, turning it into the desktop, the lamp going off as it grows.
- **Esc** puts the lamp off, walks the wall back to the picture you had in the dark, then steps it out: the old wallpaper is back.

## The bar

Along the foot, glass, a pill, `space.s8` round small controls `space.s16` apart, in the `ButtonBar`'s order: the views as `Tabs` (Picture, Desktop, Lock), Full screen (a Text button), then Cancel (Text), Wake or Rest on the Lock view only (Text), and Apply (Filled) last. On the full screen the picture's name, its folder and place, its depth chip and the previous and next buttons lead it. No key is shown: each control's `Tooltip` gives its name and its key on one line. The `Shortcuts` button sits at the foot's start, its popover listing every key, grouped; `?` opens it, and Esc closes it before Esc cancels.

## Reduce motion

Every move is a fade over `short`: the room fades out and in round a step, the frame and the bar fade, the clock fades in its place, and nothing pushes or slides. The video's loop holds still.

## The real desktop in the frames

The Desktop and Lock views show the user's own windows as stills, one set shared by every frame:

- **On opening, synchronously,** the picker asks for a frozen capture of each window on the workspace (Quickshell's `ScreencopyView`, not live, placed where Hyprland says each window is): a request, under a millisecond each, so the opening move starts at once.
- **During the opening move** (about 1.1 s), the GPU copies each capture into a texture, about 1 to 3 ms a window and about 15 MB for a full-screen one at 2560 by 1440; drawing it at a frame's scale is a transform when drawn, at no further cost.
- **Until they land,** a frame shows the picture, the frame and the bar, drawn live in its colours, and each window fades in over `slow` as its still lands. The desktop the picker opens from keeps its own windows throughout.
- **Never live:** a live capture would copy every window every frame while nobody works in them. The Lock's locking plays on the same stills.
