# OrderList

A molecule: [DESIGN.md, "Levels and where they live"](../../DESIGN.md#levels-and-where-they-live) says what a molecule is. Its mockup page is the source of truth: Molecules, OrderList, in `design/mockups/`.

## `OrderList`

Things in an order that matters, each offered or not: first boot's Kotodama card lists the web search's engines in it, the order the menu offers its web rows in, an engine switched off not offered at all.

A row each, `size.control_md` (40) tall, its sides 16 in, as a list row's are, so its switch ends where theirs do: a grab handle (`swap_vert`, `type.body`'s symbol) in `on_surface_variant`, 12, the thing's name in `body`, filling, 12, a [Switch](switch.md). Rows stand back to back with a hairline in `outline_variant` between them.

- **Lifted** (mid-drag): the row on `surface_container_highest` with the raised shadow (`space.s8` down, `space.s24` blur, `surface_container_lowest` at `opacity.scrim`), over the others.
- **Disabled:** the whole list at `opacity.disabled_content`, taking no pointer, its rows kept where they are, so nothing jumps when what it depends on is turned back on.
- **Focus:** a row takes the keyboard, the cold tube round it; its switch keeps its own.

**Motion:** a row moved glides to its new place and the row it passed to its own, each over `medium` on `standard_decel`; reduce motion moves them at once.

**Keys:** Tab reaches each row, then its switch. Alt+↑ and Alt+↓ move the focused row a place, the focus moving with it. A row is dragged by its handle, a place at a time as the pointer passes a row's height.

*Not built yet: neither library has it.*
