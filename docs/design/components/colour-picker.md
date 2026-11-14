# ColourPicker

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ColourPicker`

Its mockup page is the source of truth: Molecules, ColourPicker, in `design/mockups/`.

A colour picked for something drawn over a picture: Auto, which leaves the colour to its user (the lock clock takes the picture's own tint under its digits), one of the scheme's roles as a swatch, or Custom, a colour of its own picked by its hue and its tone. It is three controls, each the control of one row of its form, which labels them: the choice, the hue track and the Tone slider. The wallpaper picker's clock editor sets the lock clock's colour with it, offering Auto, `primary`, `secondary`, `cold` and `on_surface` (named Text), and Custom.

- **The choice** is a row of picks, `space.s8` apart, `space.s4` inside a pill (`radius.full`) that is the focus tube's box; it wraps when its row is narrower. Auto, when offered, is a pressable `Chip`, `Selected` while picked and `Neutral` otherwise. A swatch is a circle `size.chip` (24) across in its role's colour, with a hairline rim of `on_surface` at `elevation.rim`; the one picked has a ring of `primary`, `stroke.thin` wide, cut from it by a `stroke.thin` gap of `surface_container_high`, the well its clock-editor row sits in. Custom, when offered, is the last swatch: every hue round a wheel at one light tone (OKLCH 0.75, chroma 0.13), with the custom colour in its middle once it is one, a quarter of the swatch in from its edge.
- **The hue track** is a pill `size.slider_compact` (24) tall, every hue across it at that same tone, a knob `size.slider_stop` (6) across with a `stroke.thin` ring of `surface` round the hue picked, and the hue in degrees in `label_number` at the far end, `space.s12` in, dark in both modes (`surface` by night, `on_surface` by day). Its focus tube is a pill on its box's edge.
- **Tone** is a compact `Slider` with the `brightness_medium` symbol, from OKLCH lightness 0.30 to 0.97, its value a percentage of lightness; a custom colour's chroma is 0.12 at every hue and tone, so every hue stays inside sRGB at the light tones a colour over a picture takes. Before a custom colour is picked, the two start at a light blue (hue 250, lightness 0.85).

**Keys.** The choice is one stop for Tab: Left and Right (h and l) pick the pick before or after, stopping at either end, and the keyboard stays on the choice. Hue and Tone are stops only while Custom is picked: the hue track's Left and Right step a twentieth of the wheel, a degree with Shift, and Home and End go to either end; Tone takes a slider's keys. A click picks; a drag on the hue track follows the pointer.

**States:** rest, the pick ringed (or Auto `Selected`); lit, the choice's or the hue track's focus tube in the accent, with the signs off its glass; pressed, Auto's chip pressed (a swatch has no pressed look); and disabled: every pick at `opacity.disabled_content`, the wheel's chroma lowered by the same share, and the hue track and Tone disabled too, no stop for Tab. Hue and Tone are disabled whenever Custom isn't picked.

*Not built yet: no library has it.*
