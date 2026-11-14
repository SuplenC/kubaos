# IconButton and Hint

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `IconButton`

Rust `button::icon_button(glyph, label)`; QML `IconButton`.

## `Hint`

Rust `hint::hint`, `hint::hint_at_start`; QML `Hint`, `held: "start"`.

- **`IconButton` is a circle as tall as the label button of its size** (32, 40, 48), in all four kinds. **Its label is a required argument.** It is the accessible name, and shows in a `Hint` while the pointer is over it or the keyboard is on it: a bubble in the inverse colours, `label` type, padding 8 × 4, the small radius (4), centred under the control, 8 below it. In QML the bubble floats over the window, but only while its control shows whole: a control a scroll has cut, or scrolled out of view, takes its hint with it, so it never floats over what the scroll shows instead. An icon alone says nothing out loud, so an icon button can't be built without its label.
- **A hint is centred under its control, or held to its start** (`hint_at_start`, `held: "start"` in QML): its left edge on the control's, 8 below it. A control that starts something wider than itself takes it, so its hint reads from where it does: a counter's name, which a double-click renames, says so under the name's first letter rather than under the middle of a name that may run half the row. Either way it is kept inside the window.
- A button's icon is its label's style's (see [DESIGN.md, "Icons by type"](../../DESIGN.md#icons-by-type)); an icon button takes the icon of the label button of its size.
- **An icon button a `Tooltip` explains shows the tooltip instead of its hint** (`IconButton::explained`, `explained` in QML): the card already names it, and a bubble under it as well would say the name twice, below a control that may sit at the screen's foot. The notifications panel's keys button is one.
