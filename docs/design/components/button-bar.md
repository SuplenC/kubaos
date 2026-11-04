# Button bar

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps.

## `ButtonBar`

Rust `button_bar::button_bar`; QML `ButtonBar`. Its mockup page is the source of truth: Molecules, Button bar, in `design/mockups/`.

The row of buttons that ends a surface (a dialog, a form in a well). It holds the rules of their order, kinds and spacing in one place, so every surface ends the same way:

- **Order.** The dismiss button (Cancel, `Text`) at the start; the actions at the end, the one Enter presses last. That action is `Filled`, the screen's one; any other action before it is `Tonal` (Shut down beside Restart). On a `Danger` surface it is a `Danger` button naming what it does (Erase and install). An action alone sits at the end.
- **Spacing.** Buttons are 8 apart (`space.s8`), regular (`md`) unless the surface's controls are small (`sm`).
- **No inset of its own.** It sits in its surface's padding like any other content, as wide as the room its surface gives it, a step of the surface's own spacing under what is above it (24 in a dialog).
- **A Text button hangs its padding.** A `Text` button at the bar's start or end hangs its own horizontal padding (`space.s12` at sm and md, `space.s16` at lg) outside the bar, so its label, or its leading symbol, is on the content's edge like everything above it; its lit tube and its press fill reach into the surface's padding. Every other button keeps its box on the edge, so a `Filled` or `Tonal` last button's box ends on the content's right edge. A row of buttons that ends a surface but holds more than buttons (Kōbō's footer, with its note; its rail's foot, with the battery; the setup page's actions) follows the same rule. A surface whose padding has no room for it keeps the button's box on the edge instead: a card (its 16 would leave the box 4 from its edge) and a banner (its padding is its rail's lane). Stacked, nothing hangs.
- **Variants.** Dismiss and one action; dismiss and two actions; one action alone; a `Danger` action.
- **States.** None of its own: each button keeps its own.
- **A small room.** Where its buttons don't fit on one line (a narrow room, a long translation), they stack 8 apart, each the bar's full width: the one Enter presses on top, the dismiss button at the bottom. They go back to one line as soon as they fit. Tab keeps its order, dismiss first, so stacked it runs from the bottom up.

**The keyboard.** Tab and Shift+Tab go through its buttons in order, and Enter or Space presses the one the keyboard is on. Where the keyboard lands, and what Enter and Esc do on the surface, are the surface's.

**Motion.** None of its own: it arrives and leaves with its surface, and stacking cuts from one line to the stack as the room narrows.

*Not built yet: neither library has a `ButtonBar`; a dialog lays its buttons out itself, as one line, its Cancel's box on the content's edge. The mockup draws the rule.*
