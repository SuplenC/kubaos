# Card and ChoiceCard

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Card`

Rust `card::card`; QML `Card`. Its mockup page is the source of truth: Molecules, Card, in `design/mockups/`.

A surface inside the page holding a group that belongs together but isn't a list, raised one step on the ladder (`docs/DESIGN.md`, "Elevation"): `surface_container_high` by day, `surface_container_highest` at night, step 1's shadow and the rim inside its edge. Its corners are the surface radius (24): it holds content, as a panel and a well do. `space.s16` inside, as wide as its room; in a grid it takes its row's height, so cards side by side end together.

What it holds runs top to bottom `space.s16` apart: a header (a symbol at `subheading`'s 24, its title in `subheading` `space.s8` after it, a `small` line in `on_surface_variant` `space.s4` below), the body, and a `ButtonBar` at its foot, its action `Tonal`. A card or a well is never inside another; a control in a card keeps its own corner, and a shape filling the card less its padding takes the nested corner (a picture `space.s16` in, 8).

**States:** a plain card takes no input and has none. A pressable card is one press target as a whole and holds no controls: rest, lit (the accent's tube inside its edge, the glow outside at night), pressed (the tube white-hot, the glow brighter, by day `opacity.press_tint` of the accent) and disabled (its resting surface kept, what it holds at `opacity.disabled_content`, no line, no stop for Tab).

*Not built yet: both libraries draw a card on the well's surface (`surface_container_low` at `opacity.well`) with a soft hairline and no shadow, and neither has a pressable card.*

## `ChoiceCard`

Rust `card::choice_card`; QML `ChoiceCard`. Its mockup page is the source of truth: Molecules, ChoiceCard, in `design/mockups/`.

A card picked from a set, saying what each choice means: one of the set, as radios, or any of it, as checkboxes, laid in a grid of two `space.s16` apart. It is a `Card`, its header the card's, with a mark ending the title's first line: a radio's disc or a checkbox's box. Picked, its title and symbol are the accent's words (`primary_lit`, the label glow at night while the signs are on) and its mark is filled; its surface doesn't change.

**States:** rest, lit (the cold tube, since the accent already means picked), pressed (the tube white-hot, by day the cold tone's tint, its title going to the picked words) and disabled (its resting surface kept; its words, symbol and mark grey at `opacity.disabled_content`, no line, skipped by the keys). The set is one stop for Tab, on its picked card; the arrows move, and in a set of one pick as they go; Space and Enter pick or turn a card.

*Not built yet: both libraries pick a card with `secondary_container` and an accent hairline and draw no mark, light it with a state layer, and draw a disabled card on the disabled container.*
