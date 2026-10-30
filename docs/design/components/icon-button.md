# IconButton and Hint

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `IconButton`

Rust `button::icon_button(glyph, label)`; QML `IconButton`.

**Kinds:** `Filled`, `Tonal`, `Text`, `Danger`, `Success` and `Warning`, each with its [Button](button.md) kind's colours: at night Filled rests on the unlit fill (`primary_fill`) with an `on_primary_fill` symbol, Tonal on `secondary_container` with a dimmed symbol, Text with no fill and its symbol the tone unlit (`primary_rest`), and Danger, Success and Warning burn on `burn_error`, `burn_success` and `burn_warning`, always lit, with a dark symbol (`on_burn_error` and the rest).

**States** are the button's (`docs/DESIGN.md`, "States"), in every kind and size, with no state layer:

- **Rest:** unlit at night, as above.
- **Lit:** hovered or focused from the keyboard, one state, and its hint shows. Signs on, at night while the lights burn, the line inside the circle's edge burns (the core, `stroke.thin` wide, on Filled and the burning kinds; the tube, `primary` at `focus.tube`, on Tonal and Text) with the glow outside it, the only thing past the circle, and the symbol brightens as its kind's label does: Filled's with a tiny glow, Text's burning as the core with the content glow. Signs off, by day always and at night while a power change has the lights out, the same line is glass (`primary_fill_glass`, `primary_glass`, `burn_error_glass`, `burn_success_glass`, `burn_warning_glass`) with no glow and the symbol at rest.
- **Pressed:** overdriven and lit: Filled's and Tonal's fill lifts 0.035 at night (0.04 darker by day), Text's tube runs to the core (by day the tone under it at `opacity.press_tint`), the burning kinds' lit fill goes 8% further toward white (0.06 darker than rest by day); the glow is `focus.pressed_glow` times the lit one.
- **Disabled:** grey, with no tone, no line and no glow; no action and no focus. Its hint still shows while the pointer is over it.

**Motion.** Lighting up, going back to rest and pressing are fades (`short`, and `micro` for the press); the hint fades in and out over `short`. The flicker is for power changes only: in a theme change and dark ⇄ light a lit icon button flickers to its signs-off look and back, and one at rest just recolours.

*Not built yet: the libraries draw hover and press as state layers, strike focus, light `Danger` in the accent on `error` and have no `Success` or `Warning` kind; the mockups' IconButton page (`design/mockups/src/pages/atoms/icon-button.ts`) draws the rule.*

## `Hint`

Rust `hint::hint`; QML `Hint`.

- **`IconButton` is a circle as tall as the label button of its size** (32, 40, 48), in all six kinds. **Its label is a required argument.** It is the accessible name, and shows in a `Hint` while the pointer is over it or the keyboard is on it: a bubble in the inverse colours, `label` type, padding 8 × 4, the small radius (4), centred under the control, 8 below it. In QML the bubble floats over the window, but only while its control shows whole: a control a scroll has cut, or scrolled out of view, takes its hint with it, so it never floats over what the scroll shows instead. An icon alone says nothing out loud, so an icon button can't be built without its label.
- **A hint isn't neon:** no line and no glow, the same bubble by night and by day. It shows while its control is lit (hovered or focused) and fades in and out with it over `short` on `standard`; it never flickers, even in a power change, where it just recolours. Over a disabled control it still shows while the pointer is there. The mockups' Hint page (`design/mockups/src/pages/atoms/hint.ts`) shows it.
- A button's icon is its label's style's (see [DESIGN.md, "Icons by type"](../../DESIGN.md#icons-by-type)); an icon button takes the icon of the label button of its size.
- **An icon button a `Tooltip` explains shows the tooltip instead of its hint** (`IconButton::explained`, `explained` in QML): the card already names it, and a bubble under it as well would say the name twice, below a control that may sit at the screen's foot. The notifications panel's keys button is one.
