# NotificationGroup

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `NotificationGroup`

QML `NotificationGroup`. Its mockup page is the source of truth: Molecules, NotificationGroup, in `design/mockups/`. *Not built yet: the shell draws it as `CardGroup` in `config/quickshell/modules/notifications`, not a library part, and casts `shadow.tucked` and `shadow.lifted` rather than the ladder's steps; those two tokens go once it moves.*

One app's notifications as one thing, in the `PopupStack`, the bell's peek and the notifications panel, each a `NotificationCard`. A single notification is the card alone.

- **At rest, a stack:** the newest card, standing for the whole group (its count chip, and its dismiss and actions take them all), the others tucked under it. One narrower edge peeks out below it for a pair, two for more, each `space.s8` lower and `space.s8` narrower each side than the one above, in `surface_container`, each card's shadow cast onto the edge under it at the ladder's step 1, cut to the edges. The edges' room is part of the group's height, so nothing overlaps.
- **A preview, under a resting pointer:** after the pointer has rested `medium` on it, up to three cards spread down into whole cards `space.s8` apart over what is behind, each with its own shadow straight under it, rising from step 1 to step 2 as it spreads; "+N more · click to open" in `small` and `primary` sits `space.s8` under them when there are more. What holds the group fades every other row toward its surface by `opacity.recede`, and those the preview covers go entirely; nothing moves. Leaving tucks the cards back. The keyboard never previews.
- **Opened:** every card takes its place, `space.s8` apart, under a heading a small control tall: the app's name in `label`, a `Chip` with the count, and Show less, a Text sm button, at its end. The group grows, so only what is below it moves; each card is then on its own.

**Motion.** The preview spreads over `medium` on `standard_decel` and tucks in over `short` on `standard_accel`; opening takes `medium` on `standard_decel`, closing `short` on `standard_accel`. Each card's face fades in once its edge has spread; the heading fades in over the second half of the opening. With reduce motion each state comes at once and the cards fade in over `short`.

**Keys**, from what holds it: → or l opens it, the keyboard on its first card; ← or h closes it; ↑ and ↓ move card to card inside it opened and leave it past its ends, ↑ into an opened group landing on its last card; Enter or Space opens a stack and unfolds an opened card; Del dismisses the card, or a stack's whole group; Esc folds an unfolded card, then closes the group. A click on the newest card or anywhere on the preview opens it; Show less, or a click on it again, closes it, and a closed group under the pointer doesn't preview again until the pointer has left.
