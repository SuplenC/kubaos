# Field, Labelled and Note

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Field`

Rust `field::field(value, placeholder)`; QML `Field`.

One line of text, 40 tall (`control_md`), the control radius (12), filling the room it is given: `surface_container_high` with an `outline_variant` hairline inside its edge. `body_large` text in `on_surface`, 12 in from each side and centred down the field. An optional 20 icon (`body_large`'s) leads it 12 in, in `on_surface_variant`, the text starting 8 past it, at 40. A placeholder (`on_surface_variant` at `opacity.placeholder_text`) is a hint, never a value that looks typed. Passwords show one bullet per character.

**States** follow the neon (`docs/DESIGN.md`, "States"), with no state layer:

- **Rest:** unlit, as above.
- **Lit:** hovered or with the keyboard in it, one state, a read-only field too. The fill rises to `surface_container_highest`. Signs on, at night while the lights burn, the tube burns over the hairline, `primary` at `focus.tube` inside the edge, with the glow outside it. Signs off, by day always and at night while a power change has the lights out, the tube is solid glass, `primary_glass`, with no glow. With the keyboard in it, it shows the caret, or the selection; hovered, never the caret.
- **Invalid:** the value is wrong. The hairline is `error` at rest; lit, the tube, the glow and the caret are the error's (`error`, `error_core` for the caret at night, `error_glass` with the signs off), and the fill stays `surface_container_high`. The reason is in the note under it.
- **Locked:** read-only: its value can be selected and copied, never edited. The fill and hairline at `opacity.locked`, lit or not, the value `on_surface_variant`. It is a stop for Tab and lights as any field does; the pointer and the keys select in it as in an editable field, and Ctrl+C copies. With the keyboard in it the caret shows, so a selection from the keys has a place to start: it strikes on with focus but never glows, since nothing is typed.
- **Disabled:** not usable now. Grey, with no tone, no line and no glow: the field `on_surface` at `opacity.disabled_container`, the value, placeholder and icon at `opacity.disabled_content`, no hover, no caret, no stop for Tab. Its value still selects with the pointer and Ctrl+C copies it, in the field's own selection: selecting is useful even where nothing can be changed. In QML it is a `status`, as `Locked` is, not `enabled: false`, which would take its input but not change its look.

A field has no pressed look: a press puts the caret where it lands.

**The caret** is one core line, the thin stroke wide (`stroke.thin`, whole device pixels) with rounded ends, 16 tall (`caret.height`) and centred on the value's 20 line, so 2 short at each end. It is centred on the boundary between the letters either side, on a whole device pixel. At night it is the tone's core (`primary_core`, `error_core` when invalid); by day it is the tone (`primary`, `error`); with the signs off (a power change's dark beat, or the signs-off look) it is the tone's glass, `primary_glass` or `error_glass`, as the line is, with no glow. It is steady: it never blinks or breathes.

At night, while you type, a glow in the tone shows round the caret's own shape, 0.25 (`caret.glow`) at its edge fading to nothing 4 out (`caret.near`), drawn as the focus glows are. It comes in over `caret_glow` (80 ms), holds while keys follow, and `caret_hold` (500 ms) after the last key fades over `caret_fade` (800 ms) to nothing. By day nothing glows and nothing changes while you type. A read-only field's caret never glows.

A selection is `primary` at `opacity.selection` behind the selected text, at the small radius (4), and hides the caret. A value longer than the field scrolls sideways to keep the caret inside its room; the text is clipped at the room.

The app owns the value and its editing: it keeps the `Caret` (where it is, what is selected), turns keys into edits, and gives the field both; the field draws the selection and the caret, scrolls the text, and turns presses and drags into `Pointer`s (a press, a double press on a word, a triple press on all of it, a drag). Real keys, a scripted tour and the tests all edit through the same code.

**Motion.** Lighting up and going back to rest are fades over `short` on `standard`; turning invalid fades the tone from the accent to the error's over `short`. The caret moves and typed text appears at once. The flicker is for power changes, and for the caret: as the keyboard comes in it strikes on through `neon.strike`'s steps over `medium`, by night and by day, while the field's line lights up with its fade. At night its glow strikes with it, then holds and fades as after typing. With the flicker off the strike is a fade over `short`; with reduced motion the caret just shows, and its glow shows and hides at once.

*Not built yet: in both libraries hovering only darkens the hairline to `outline` and the tube is the keyboard's alone, the hairline changes over `micro`, the tube strikes, a read-only field never lights and can't be selected, and the caret blinks, the line's full height in `primary`, tight against the letter, `primary` on an invalid field too; the mockups' Field page (`design/mockups/src/pages/atoms/field.ts`) draws the rule.*

## `Labelled`

Rust `field::labelled`; QML `Labelled`.

A control under its `label`-type label in `on_surface_variant`, with an optional small action at the label's right (Show, a small Text button) and an optional note under it, each 4 apart. The label row is 32 tall (`control_sm`) whether or not it has an action, so fields side by side line up.

## `Note`

Rust `field::note`; QML `Note`.

The `small` line under a field: `on_surface_variant`, or `error` when invalid. A long note breaks between words in the field's width. *Not built yet: QML's `Note` takes no room, so its line runs on past the field.*
