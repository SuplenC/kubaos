# IconButton and Hint

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `IconButton`

Rust `button::icon_button(glyph, label)`; QML `IconButton`.

**States:** resting, hovered, pressed, focused and disabled, in every kind and size; its hint shows while the pointer or the keyboard is on it.

## `Hint`

Rust `hint::hint`; QML `Hint`.

- **`IconButton` is a circle as tall as the label button of its size** (32, 40, 48), in all four kinds. **Its label is a required argument.** It is the accessible name, and shows in a `Hint` while the pointer is over it or the keyboard is on it: a bubble in the inverse colours, `label` type, padding 8 × 4, the small radius (4), centred under the control, 8 below it. In QML the bubble floats over the window, but only while its control shows whole: a control a scroll has cut, or scrolled out of view, takes its hint with it, so it never floats over what the scroll shows instead. An icon alone says nothing out loud, so an icon button can't be built without its label.
- A button's icon is its label's style's (see [DESIGN.md, "Icons by type"](../../DESIGN.md#icons-by-type)); an icon button takes the icon of the label button of its size.
- **An icon button a `Tooltip` explains shows the tooltip instead of its hint** (`IconButton::explained`, `explained` in QML): the card already names it, and a bubble under it as well would say the name twice, below a control that may sit at the screen's foot. The notifications panel's keys button is one.
