# Dialog and Scrim

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Dialog`

Rust `dialog::dialog`, `dialog::prompt`; QML `Dialog`. Its mockup page is the source of truth: Molecules, Dialog and Scrim, in `design/mockups/`.

A question over a dimmed screen: one width (500, `dialog`), padding 24 on every side, under its buttons too, the dialog radius (16, `radius.dialog`), raised to the ladder's step 3 (`surface_container_high` by day, `surface_container_highest` at night, the step's shadow and the rim). A 24 icon (`heading`'s) in `primary` beside a `heading` title, 12 apart; the `body` in `on_surface_variant` 12 under the title and in line with it, beside the icon; what it asks for 16 under the body, in line with it too; and its [button bar](button-bar.md) 24 under the last, across its whole width: the dismiss button (Cancel, `Text`) on the left, the actions on the right, the one Enter presses last (the screen's one `Filled`, a `Tonal` other way out before it), 8 apart, all regular. Tone `Normal` or `Danger`, built the same: a `Danger` dialog shows the warning sign in `error`, an edge of `error` at `opacity.danger_edge` over the rim, and a `Danger` action naming what it does. A prompt asks for something only the user can type (a password): a `Normal` dialog with a `Labelled` field 16 under the body, Enter in the field doing what its action does; a wrong answer is the field's `Invalid` state and its note, and the dialog stays. In QML it is a `Dialog` with the field as its child.

**The keyboard.** Opening, it takes the keyboard: onto its field, else Cancel on a `Danger` dialog, else its last action. Tab and Shift+Tab go round its field and buttons and never leave it; Esc does what Cancel does; closing gives the keyboard back to what opened it.

**Motion.** It arrives over `long` on `emphasized_decel`, fading in as it rises 16 into place a whole device pixel at a time, as its scrim fades in; the two fade out together over `medium` on `emphasized_accel`. With reduce motion both fade, capped at `short`, with no rise.

## `Scrim`

Rust `dialog::scrim`; QML `Scrim`.

Everything behind a dialog, dimmed (`scrim` at `opacity.scrim`), with the dialog in its middle on whole device pixels. It takes every press, hover and scroll, and a press on it does nothing.

*Not built yet: both libraries draw the dialog 32 inside at the surface radius (24), with a 32 icon and a `headline` title over the body and the buttons 32 under it, and lay its buttons out themselves rather than in a button bar; they cast `shadow.dialog` with an edge of `primary` at `opacity.dialog_edge` instead of the ladder's step 3 and its rim, and neither moves as it opens or closes. The mockup draws the rule.*
