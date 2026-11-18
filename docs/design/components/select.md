# Select

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

The mockup's Select page (`design/mockups/src/pages/molecules/select.ts`) is the source of truth; this page sums it up.

## `Select`

Rust `select::select`; QML `Select`.

One of a list too long for radio rows or tabs. A row of tabs that would run past its room is a Select instead. The clock's holidays.

**Shut**, it has a field's look: 40 (`control_md`) tall at the control radius, filling its room, `surface_container_high` with a field's sunken edge, a hairline of `outline` inside its edge, 12 inside. The pick is `body_large` in `on_surface` on one line, ending in an ellipsis where it doesn't fit; an optional 20 symbol (`icon`) leads it and a chevron ends it, both `on_surface_variant`, 8 from the words. With nothing picked it shows its placeholder in `on_surface_variant`. **Sizes:** md (`Md`, above) is a form's; sm (`Sm`) is a list row's control (`List`'s rule, [list.md](list.md)), a small field's look: 32 tall (`control_sm`), the pick in `body`, its symbol and chevron `body`'s 20, 8 from the words, 12 inside. Open, its list is the same at both sizes, its rows already `control_sm` with `body` words, so a small select's open list continues its value row without a step.

*Not built yet: both libraries draw the shut select's hairline in `outline_variant`, and its edge open in `primary`; the mockup draws the rule.*

**Open**, the select's own box grows into its list: one surface, no gap and no second popover.

- The box rises to the ladder's step 2, a menu's (`DESIGN.md`, "Elevation"): the step's shadow and rim, `surface_container_high` by day and `surface_container_highest` at night. Its hairline goes, its tube goes out and the chevron turns up.
- The value row stays where it was, at the top, and the list is under it. A rule in a divider's soft outline (`outline_variant` at `opacity.outline_soft`), 12 in from the sides, separates them.
- The options are menu items (`menu.md`): 32 (`control_sm`) tall, 4 inside the surface, their words `body`, lined up with the value row's start, 12 in, at the row radius (8), the control radius less 4. There is no check. The open list is built as a popover's menu is, the one rule for every raised list.
- An option may carry a short description: its menu item's line under its words in `small`, `on_surface_variant`, wrapped to at most two lines and cut there with an ellipsis after the last whole word. Every row of that list is then at least a two-line row's height (`row_two_line`, 56), and a row whose description wraps is a three-line row's (`row_three_line`, 72); the list's eight in view are measured as laid out (`two-line`). A Bluetooth headset's codecs, each saying what it is good for. The shut select shows the pick's words alone.
- An option may end in a 16 symbol of its own, in the same muted colour (the menu item's `trailing`): in the Audio panel's default-device Select, a choice that brings no mic ends in `mic_off`. It too shows only in the list.
- The highlighted row is a lit menu item, `secondary_container` with `on_secondary_container`, fading over `short`. The pointer moving onto a row and the keyboard move one highlight.
- The picked option's words are `primary_lit`, a picked tab's, with the label glow round them at night while the signs are on. Highlighted, the pick takes the highlight's colours.
- Past eight options the list scrolls, with no bar of its own. The surface is the value row and the list's visible part, never the whole list; the value row never scrolls. It opens scrolled so the pick has three rows above it where the list allows, and the highlight keeps itself in view, 4 clear of the list's edge.
- Too near the bottom of the room that clips it, or of the window, for its list, it grows upward: the value row stays put and the list opens above it, the rule between them.

**Inline**, a choice inside a line of text (`look: inline`: an app's device under its name in the Audio panel): no border and no box, the pick in `small` and `on_surface_variant` with a `small` chevron 4 after it. Under the pointer or the keyboard only the words and the chevron turn `primary_lit`, a lit Text button's words, with no tube and no fill. Open, the chevron turns up and its list is a `Menu` on a popover raised to step 2, 4 under the words (over them where the room below is too short), its options' words in line with the pick's; it fades in over `short` and out over `micro`, as a popover does. *Not built yet: neither library has the inline look.*

**Motion.** It opens over `medium` on `emphasized_decel`: the surface grows from the shut box to its open height, clipped to it, while its shadow, rim and fill come up and the chevron turns. It folds back over `short` on `emphasized_accel`. A pick shows the value at once and the row pressed over `micro`, then the surface folds. With reduced motion it opens and shuts at once, the chevron with it.

**Keys.** Shut: Enter, Space, ↓ or ↑ opens it with the highlight on the pick. Open: ↑ ↓ move the highlight, stopping at either end; Home and End go to the first and the last; a letter jumps to the next option starting with it; Enter or Space picks; Esc closes without picking; Tab closes and moves on. A press outside closes it. The keyboard stays on the select throughout: the list is no stop of its own. Whatever closes or hides what holds the select (its panel closed by any route, its page swapped out) closes it too: in QML the owner calls `dismiss()` as its panel goes, and a select hidden closes itself. In iced the app opens the select and draws its list.

**States:** rest, lit (hovered or with the keyboard on it: the fill rises to `surface_container_highest` and the tube burns over the hairline, with the glow at night), pressed (the glow at `focus.pressed_glow`), open, and disabled (grey as a disabled field, no action, `enabled: false`).
