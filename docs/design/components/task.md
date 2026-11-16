# Task

A molecule: [DESIGN.md, "Levels and where they live"](../../DESIGN.md#levels-and-where-they-live) says what a molecule is. Its mockup page is the source of truth: Molecules, Task, in `design/mockups/`.

## `Task`

One thing to try, ticked once it is done: first boot's Kotodama card lists its three tries in it (Open it; Try wifi, then Enter; Try 12*7). Nothing it asks is required; it says what has been tried.

A card `size.row_two_line` (56) tall at the least, 12 in above and below and 16 at its sides, at the content corner (16), on `surface_container` with a hairline edge in `outline_variant`: a round tick `size.check` (20) wide, 12, then its title in `body_strong` over a line in `small` in `on_surface_variant`. Words to type in the title are set in `mono_large` in `primary`, 4 after the words before them.

- **To do:** the tick an empty ring `stroke.thin` thick in `outline`.
- **Done:** the card lit as a picked card is, the tube `focus.tube` in `primary` with its near glow at night and none by day, the accent's tint (`opacity.tint`) in it; the tick filled `primary_fill`, its check in `on_primary_fill`.

**Motion:** once done, the card lights over `medium` on `standard`, and the tick pops in from 0.4 of its size over `medium` on `pop`; reduce motion shows it done at once.

**Keys:** it takes no focus. A press starts what it asks for (opens the menu, types the words); the page's own keys do the same.

*Not built yet: neither library has it.*
