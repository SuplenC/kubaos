# Field, Labelled and Note

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Field`

Rust `field::field(value, placeholder)`; QML `Field`.

One line of text, 40 tall (`control_md`), the control radius (12), a hairline outline, which the focus tube takes over while it is focused. `body_large` text. An optional 20 icon leads it, the text starting at 40. A placeholder is a hint, never a value that looks typed. States: resting, hovered, focused (the caret, and the tube on its edge), `Invalid` (error colour, the reason in the note), `Locked` (read-only, dimmed, never focused) and `Disabled` (not usable now: the value, placeholder and icon at the disabled opacity, the field the disabled container, no hover, no caret, and no stop for Tab). In QML it is a `status`, as `Locked` is, not `enabled: false`, which would take its input but not change its look. Passwords show one bullet per character. The app owns the value and its editing: it keeps the `Caret` (where it is, what is selected), turns keys into edits, and gives the field both; the field draws the selection and a caret the thin stroke wide that blinks (solid for 530 ms after it moves, so it never hides while you type), scrolls the text sideways to keep the caret in view, and turns presses and drags into `Pointer`s. Real keys, a scripted tour and the tests all edit through the same code.

## `Labelled`

Rust `field::labelled`; QML `Labelled`.

A control under its `label`-type label, with an optional small action at the label's right (Show, a small button) and an optional note under it. The label row is 32 tall whether or not it has an action, so fields side by side line up.

## `Note`

Rust `field::note`; QML `Note`.

The `small` line under a field: muted, or the error colour when invalid.
