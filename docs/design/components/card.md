# Card and ChoiceCard

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Card`

Rust `card::card`; QML `Card`.

A surface inside the page: the well's surface, a soft hairline, the surface radius (24), 16 inside.

**States:** none: a plain card takes no input.

## `ChoiceCard`

Rust `card::choice_card`; QML `ChoiceCard`.

One of a few cards to pick from, at the surface radius (24). Picked: `secondary_container` and an accent hairline. Its focus tube is cold, since the accent already means picked.

**States:** resting, hovered, picked, focused (cold), and disabled (no `on_pick`, `enabled: false`): the card the disabled container with the plain hairline, picked or not, what it holds at the disabled opacity, no hover, and no stop for Tab. In iced the owner builds what it holds from `look.faded(Opacity::DisabledContent.value())`, since iced can't dim what it is given.
