# Dialog and Scrim

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Dialog`

Rust `dialog::dialog`, `dialog::prompt`; QML `Dialog`.

A question over a dimmed screen: one width (500, `dialog`), padding 32, the surface radius (24), the dialog shadow. A 32 icon (`headline`'s) and a `headline` title, the `body` 12 under it, the buttons 32 under the last: the dismiss button (Cancel) on the left, the actions on the right, the one Enter presses last. Tone `Normal` or `Danger`. A prompt asks for something only the user can type (a password): a `Normal` dialog with a `Labelled` field 16 under the body, Enter in the field doing what its action does. In QML it is a `Dialog` with the field as its child.

## `Scrim`

Rust `dialog::scrim`; QML `Scrim`.

Everything behind a dialog, dimmed (`opacity.scrim`).
