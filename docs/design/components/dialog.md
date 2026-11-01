# Dialog and Scrim

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Dialog`

Rust `dialog::dialog`, `dialog::prompt`; QML `Dialog`. Its mockup page is the source of truth: Molecules, Dialog and Scrim, in `design/mockups/`.

A question over a dimmed screen: one width (500, `dialog`), padding 24 on every side, under its buttons too, the dialog radius (16, `radius.dialog`), raised to the ladder's step 3 (`surface_container_high` by day, `surface_container_highest` at night, the step's shadow and the rim). A 24 icon (`heading`'s) in `primary` beside a `heading` title, 12 apart; the `body` in `on_surface_variant` 12 under the title and in line with it, beside the icon; what it asks for 16 under the body, in line with it too; and its [button bar](button-bar.md) 24 under the last, across its whole width: the dismiss button (Cancel, `Text`) on the left, the actions on the right, the one Enter presses last (the screen's one `Filled`, a `Tonal` other way out before it), 8 apart, all regular. Tone `Normal` or `Danger`, built the same: a `Danger` dialog shows the warning sign in `error`, an edge of `error` at `opacity.danger_edge` over the rim, and a `Danger` action naming what it does. A prompt asks for something only the user can type (a password): a `Normal` dialog with a `Labelled` field 16 under the body, Enter in the field doing what its action does; a wrong answer is the field's `Invalid` state and its note, and the dialog stays. In QML it is a `Dialog` with the field as its child.

**Height.** It is never shorter than `aspect.dialog`, 5:2, of its width: 200, on the 4 px grid. A dialog that asks little (one line and its buttons) stands as a card, not a strip: its words stay at the top, its button bar at its foot, 24 from the bottom as always, and the room it gains goes between them. A dialog with no buttons (a progress) keeps it too, its words at the top. Taller than that it is as tall as what it holds, the bar 24 under the content. It is never taller than its room (the screen, or the window it opens over) less 48 (`space.s48`) above and below, so the dimmed screen always shows round it. Past that only the middle scrolls (the body and what it lists or asks for), 12 under the title, with no bar of its own: the icon and title stay at the top and the button bar at the foot. While more of the middle is below, a hairline divider in the soft outline (`outline_variant` at `opacity.outline_soft`) crosses the dialog's padding at the middle's foot, 24 above the bar, the middle scrolling under it; while more is above, the same one sits at its top, 12 under the title. Each fades over `short` on `standard` as that end is reached. The middle reaches into the dialog's padding at the sides, so a field's glow in it isn't cut.

**The keyboard.** Opening, it takes the keyboard: onto its field, else Cancel on a `Danger` dialog, else its last action. Tab and Shift+Tab go round its field and buttons and never leave it, scrolling a field in a scrolled middle into view 12 clear of its edges; from a button, Up and Down scroll a middle too tall to show whole by a line, Page Up and Page Down by what it shows less a line; Esc does what Cancel does; closing gives the keyboard back to what opened it.

**Motion.** It arrives over `long` on `emphasized_decel`, fading in as it rises 16 into place a whole device pixel at a time, as its scrim fades in; the two fade out together over `medium` on `emphasized_accel`. With reduce motion both fade, capped at `short`, with no rise.

## `Scrim`

Rust `dialog::scrim`; QML `Scrim`.

Everything behind a dialog, dimmed (`scrim` at `opacity.scrim`), with the dialog in its middle on whole device pixels. It takes every press, hover and scroll, and a press on it does nothing.

*Not built yet: both libraries draw the dialog 32 inside at the surface radius (24), with a 32 icon and a `headline` title over the body and the buttons 32 under it, and lay its buttons out themselves rather than in a button bar; they cast `shadow.dialog` with an edge of `primary` at `opacity.dialog_edge` instead of the ladder's step 3 and its rim, neither moves as it opens or closes, and neither keeps a least or a most height or scrolls its middle. The mockup draws the rule.*
