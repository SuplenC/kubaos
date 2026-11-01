# Button bar

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps.

## `ButtonBar`

Rust `button_bar::button_bar`; QML `ButtonBar`. Its mockup page is the source of truth: Molecules, Button bar, in `design/mockups/`.

The row of buttons that ends a surface (a dialog, a form in a well). It holds the rules of their order, kinds and spacing in one place, so every surface ends the same way:

- **Order.** The dismiss button (Cancel, `Text`) at the start; the actions at the end, the one Enter presses last. That action is `Filled`, the screen's one; any other action before it is `Tonal` (Shut down beside Restart). On a `Danger` surface it is a `Danger` button naming what it does (Erase and install). An action alone sits at the end.
- **Spacing.** Buttons are 8 apart (`space.s8`), regular (`md`) unless the surface's controls are small (`sm`).
- **No inset of its own.** It sits in its surface's padding like any other content, as wide as the room its surface gives it, a step of the surface's own spacing under what is above it (24 in a dialog).
- **Variants.** Dismiss and one action; dismiss and two actions; one action alone; a `Danger` action.
- **States.** None of its own: each button keeps its own.
- **A small room.** Where its buttons don't fit on one line (a narrow room, a long translation), they stack 8 apart, each the bar's full width: the one Enter presses on top, the dismiss button at the bottom. They go back to one line as soon as they fit. Tab keeps its order, dismiss first, so stacked it runs from the bottom up.

**The keyboard.** Tab and Shift+Tab go through its buttons in order, and Enter or Space presses the one the keyboard is on. Where the keyboard lands, and what Enter and Esc do on the surface, are the surface's.

**Motion.** None of its own: it arrives and leaves with its surface, and stacking cuts from one line to the stack as the room narrows.

*Not built yet: neither library has a `ButtonBar`; a dialog lays its buttons out itself, as one line.*
