# Layout

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Page`

Rust `layout::page`; QML `Page`.

A whole page on the surface, its content the container padding in at the sides, 32 below the top and 24 above the foot (see [DESIGN.md, "The container"](../../DESIGN.md#the-container)).

## `Container`

Rust `layout::container`; QML `Container`.

A panel's body or a toolbar band: its content the container padding in at the sides, as tall as the content, which keeps its own spacing above and below.

## `Header`

Rust `layout::header`; QML `Header`.

A page's Japanese accent (`accent`), title (`title`), and one lede line (`body`) 8 below.

## `Section`

Rust `layout::section`; QML `Section`.

A titled part of a page: `heading`, an optional `body` line 4 under it, the content 16 below.

## `Stack`

Rust `layout::stack`; QML `Stack`.

Children one above the other, a spacing token apart; start-aligned, or centred on whole device pixels.

## `Row`

Rust `layout::row`; QML `Row`.

Children side by side, a spacing token apart; top, centre or bottom.

## `Grid`

Rust `layout::grid`; QML `Grid`.

Equal columns, a spacing token apart both ways, edges on device pixels.

## `Split`

Rust `layout::split`; QML `Split`.

Columns of set weights (`[1, 3]` is a label column a quarter wide).

## `Divider`

Rust `layout::divider`; QML `Divider`.

A hairline across, in the soft outline.

## `Fill`

Rust `layout::fill`; QML `Fill`.

Room that pushes what follows to the row's far end.

## `Gap`

Rust `layout::gap`; QML `Gap`.

Empty room of one spacing token, where a stack needs one larger step.

## `Flow`

Rust `layout::flow`; QML `Flow`.

Children side by side, a spacing token apart both ways, a new line started where the next doesn't fit: chips, tags, checkboxes. Its lines tell the geometry check nothing; each child still says what it is.

A `Grid` row is as tall as its tallest child, and a child that fills its height is stretched to the row, so cards side by side end together (`card::card_sized`). Rows may all be as tall as the tallest (`uniform_rows`): Kōbō's four summary cards are one size. A grid can fill all the height there is (`fill_height`), sharing it between its rows: columns of lists that run to the page's foot; in a scroll, where the height has no end, each row keeps its own. `layout::page_clear` is a `Page` with nothing behind it, over a surface of its own (Kōbō's glass panel, a pane beside a rail).

Gaps, paddings and fixed sizes take only `Space`, `Size` and `Inset` tokens, so a view built from these can't leave the scale. Text in a type style is `Text` (`text::text`); a symbol is `Icon` (`icon::icon`). Text in a box of fixed height (a row's title, a card's name) is one line that ends in an ellipsis where it doesn't fit (`text::line`), so a long name never wraps out of its box. In the shell, where a window's title or a notification's body must keep to its lines, QML's `Text` takes `maximumLines`: the last line it keeps ends in an ellipsis at its room. It also takes `markup`, the same words with runs of them in other colours (matched characters, lit as Kotodama marks them), drawn in place of the words while they keep to one line.
