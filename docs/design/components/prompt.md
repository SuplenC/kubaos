# Prompt

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Prompt`

Rust `prompt::prompt(label, field)`; QML `Prompt`.

One value asked for in a popover: a counter's new name or value, the rule it resets by. Its label in `label` type with an Enter keycap at the label row's end, the `Field` under it, and the Field's `Note` under that when there is something to say: the library's `Labelled`, `Field`, `Keycap` and `Note`, nothing of its own. It is a menu wide (240, `menu`), as tall as its label row (32, `control_sm`), Field (40) and Note, 4 apart.

- **Floating** (`show(anchor, held)` in QML): on a `Popover` of its own, 12 inside, `space.s8` under the text it changes, its label and Field lined up with that text's start, or with its end (`held: "end"`) for text at a panel's end (a count), so the prompt stays on the panel.
- **In a menu** (`menu`; `prompt(…).in_menu()`, `Prompt { inMenu: true }`): inside a menu's popover between its separators (Auto reset's rule), with no popover of its own, so its Field's edges land where a floating one's do, 12 from the popover's edge.

**States.** It opens with the keyboard in its Field and the whole value selected: there is no resting state. Typed empty, its Note says nothing, or what the value reads as (`note`: "Resets every day at 9:00."). Enter on a value it can't take turns the Field `Invalid` over `short`, the reason in its Note at once (`check` gives it: "A whole number, 0 or more."), and it stays open; once the value is one it takes, the Field is normal again.

**Keyboard.** Enter takes the value (`saved`), never on a value it can't take. Esc cancels (`cancelled`); in a menu, its owner goes back to the menu. Tab and Shift+Tab stay in its Field while it floats, its one stop; in a menu they go on through the menu. ← →, Home, End and Ctrl+A are the Field's own. Closed, the keyboard goes back to what opened it, which the owner does.

**Pointer.** A click outside a floating prompt cancels it; a press in its Field places the caret.

**Motion.** Floating, it arrives as a popover does, fading in over `short` on `emphasized_decel` as its Field's caret strikes on, and leaves over `micro` on `emphasized_accel`, saved or cancelled alike; a saved count then rolls (`RollingNumber`). Reduce motion shows it and takes it away at once.

**What is the app's.** In `kubaos-ui` the app holds the value, checks it, takes the keys and floats the popover at its anchor, as with every field and popover; the prompt draws what it is given. `KubaOS.Components`' `Prompt` does all of it itself: `check`, `note`, `saved`, `cancelled`, `show` and `hide`.
