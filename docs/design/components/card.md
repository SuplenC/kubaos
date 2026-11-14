# Card and ChoiceCard

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Card`

Rust `card::card`; QML `Card`. Its mockup page is the source of truth: Molecules, Card, in `design/mockups/`.

A region of the page holding a group that belongs together but isn't a list. It is part of the page, so it lies flat on it, with no shadow and no rim (`docs/DESIGN.md`, "Elevation"): its fill is one container role above the surface it sits on, `surface_container` on a window's `surface`, `surface_container_high` on a panel's `surface_container`, and so on up; its edge is the sunken soft hairline, `outline_variant` at `opacity.outline_soft`, the list well's line, which holds the edge by day where the two tones are close. Being dragged is the one time a card is raised; nothing drags a card yet. Its corners are the content radius (16, `radius.content`): a box of content inside a window or a panel, as a list well and a dialog are. `space.s16` inside, as wide as its room; in a grid it takes its row's height, so cards side by side end together.

What it holds runs top to bottom `space.s16` apart: a header (a symbol at `subheading`'s 24, its title in `subheading` `space.s8` after it, a `small` line in `on_surface_variant` `space.s4` below), the body, and a `ButtonBar` at its foot, its action `Tonal`. A card or a well is never inside another; a control in a card keeps its own corner, and a picture filling the card less its padding takes the row radius (8, `radius.row`), since the card's 16 less its 16 would leave no corner.

**States:** a plain card takes no input and has none. A pressable card is one press target as a whole and holds no controls: rest, lit (the accent's tube inside its edge, the glow outside at night), pressed (the tube white-hot, the glow brighter, by day `opacity.press_tint` of the accent over its fill) and disabled (its resting fill and edge kept, what it holds at `opacity.disabled_content`, no line, no stop for Tab).

*Not built yet: both libraries draw a card at the surface radius (24) on the well's fill (`surface_container_low` at `opacity.well`) rather than the container above its surface, and neither has a pressable card.*

## `ChoiceCard`

Rust `card::choice_card`; QML `ChoiceCard`. Its mockup page is the source of truth: Molecules, ChoiceCard, in `design/mockups/`.

A card picked from a set, saying what each choice means: one of the set, as radios, or any of it, as checkboxes, laid in a grid of two `space.s16` apart. It is a `Card`, flat on the page on the card's fill and edge, its header the card's, with a mark ending the title's first line: a radio's disc or a checkbox's box. Picked, its title and symbol are the accent's words (`primary_lit`, the label glow at night while the signs are on) and its mark is filled; its surface doesn't change.

**States:** rest, lit (the cold tube, since the accent already means picked), pressed (the tube white-hot, by day the cold tone's tint, its title going to the picked words) and disabled (its resting surface kept; its words, symbol and mark grey at `opacity.disabled_content`, no line, skipped by the keys). The set is one stop for Tab, on its picked card; the arrows move, and in a set of one pick as they go; Space and Enter pick or turn a card.

*Not built yet: both libraries pick a card with `secondary_container` and an accent hairline and draw no mark, light it with a state layer, and draw a disabled card on the disabled container.*
