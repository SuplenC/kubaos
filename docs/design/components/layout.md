# Layout

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Page`

Rust `layout::page`; QML `Page`.

A whole page on the surface, filling its room both ways, its content the container padding in at the sides (16 under 640 wide, 24 from 640, 32 from 1024, by the page's own width), 32 below the top and 24 above the foot (see [DESIGN.md, "The container"](../../DESIGN.md#the-container)). `layout::page_clear` is the same with nothing behind it.

## `Container`

Rust `layout::container`; QML `Container`.

A panel's body or a toolbar band: its content the container padding in at the sides, as tall as the content, which keeps its own spacing above and below.

## `Header`

Rust `layout::header`; QML `Header`.

A page's Japanese accent (`accent`, in `primary`), title (`title`), and its subtitle (`body`, `on_surface_variant`) 8 below; the title and the subtitle wrap between words.

**A header has a subtitle always or never.** A surface decides once whether its headers have one (every Settings page, every Kōbō step, every right-edge panel does). Given, the line keeps its height even with no words, so the title, the subtitle and the buttons beside them never move. The subtitle is where the surface's attitude goes, a pun in `docs/BRAND.md`'s voice; counts and status stay in the body. Buttons beside a header sit at its end, 16 from the title's room and 8 apart, centred on the title-and-subtitle block, not on the accent. *Not built yet: both libraries always draw the line (an empty one keeps its height), so neither has a header without one, and neither takes buttons; the mockups draw the rule.*

## `Section`

Rust `layout::section`; QML `Section`.

A titled part of a page: `heading`, an optional `body` line 4 under it in `on_surface_variant`, the content 16 below. A group with no name is no section; a section is never a card, and a card never holds one.

## `Stack`

Rust `layout::stack`; QML `Stack`.

Children one above the other, a spacing token apart; start-aligned, or centred on whole device pixels, an odd spare pixel after the child. A child may fill the width, or the height the stack has left.

## `Row`

Rust `layout::row`; QML `Row`.

Children side by side, a spacing token apart; top, centre or bottom. One line that never wraps: what doesn't fit is a `Flow`'s, or ends in an ellipsis.

## `Grid`

Rust `layout::grid`; QML `Grid`.

Equal columns, a spacing token apart both ways, edges on device pixels.

## `Split`

Rust `layout::split`; QML `Split`.

Columns of set weights (`[1, 3]` is a label column a quarter wide).

## `Divider`

Rust `layout::divider`; QML `Divider`.

A hairline across, in the soft outline, between two parts a gap alone can't part: a window's bar and its body. The stack's gap keeps it clear either side.

## `Fill`

Rust `layout::fill`; QML `Fill`.

Room that pushes what follows to the row's far end. It never grows down.

## `Gap`

Rust `layout::gap`; QML `Gap`.

Empty room of one spacing token both ways, where one step of a stack wants to be larger than its gap. Where every step is the same, the stack's gap is the answer; where a whole box wants room round it, its padding is.

## `Flow`

Rust `layout::flow`; QML `Flow`.

Children side by side, a spacing token apart both ways, a new line started where the next doesn't fit: chips, tags, checkboxes. Its lines tell the geometry check nothing; each child still says what it is.

A `Grid` row is as tall as its tallest child, and a child that fills its height is stretched to the row, so cards side by side end together (`card::card_sized`). Rows may all be as tall as the tallest (`uniform_rows`): Kōbō's four summary cards are one size. A grid can fill all the height there is (`fill_height`), sharing it between its rows: columns of lists that run to the page's foot; in a scroll, where the height has no end, each row keeps its own. `layout::page_clear` is a `Page` with nothing behind it, over a surface of its own (Kōbō's glass panel, a pane beside a rail).

Things whose edges line up down the page (pictures, cards, labels and values) are a `Grid` or a `Split`; things of their own widths that only wrap are a `Flow`; one line is a `Row`. A layout puts its children in reading order, so Tab goes through them as they are seen: a stack top to bottom, a row from its start, a grid row by row. The mockups' Layout pages show each with its measures, and Settings' Appearance page assembled from them at every window size.

Gaps, paddings and fixed sizes take only `Space`, `Size` and `Inset` tokens, so a view built from these can't leave the scale. Text in a type style is [`Text`](text.md); a symbol is [`Icon`](icon.md).
