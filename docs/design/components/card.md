# Card and ChoiceCard

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Card`

Rust `card::card`; QML `Card`.

A surface inside the page: the well's surface, a soft hairline, the surface radius (24), 16 inside.

## `ChoiceCard`

Rust `card::choice_card`; QML `ChoiceCard`.

One of a few cards to pick from, at the surface radius (24). Picked: `secondary_container` and an accent hairline. Its focus tube is cold, since the accent already means picked.
