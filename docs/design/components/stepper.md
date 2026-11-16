# Stepper

A molecule: [DESIGN.md, "Levels and where they live"](../../DESIGN.md#levels-and-where-they-live) says what a molecule is. Its mockup page is the source of truth: Molecules, Stepper, in `design/mockups/`.

## `Stepper`

A page's own few steps (two to five) on one line under its title: where the reader is among them, and the way back a step. First boot's Make it yours is one card of the rail and three steps of its own (What for, Your apps, Go); its Stepper says which. The pages of a flow are a [StepRailPage](step-rail-page.md)'s rail, never a Stepper, so the two never say the same thing.

Each step is a bead `type.body`'s symbol wide (20) holding its number in `label_number`, then its label in `label`, 8 after it; the steps sit 24 apart and wrap onto a second line, 8 under the first, where the room ends.

- **Not reached:** the bead a ring `stroke.thin` thick in `outline`, its number and label in `on_surface_variant`.
- **Current:** the ring and the number `primary`, glowing near (`focus.near`) at night as the rail's lit symbols do, none by day; the label `primary_lit`.
- **Done:** the bead filled `primary_fill`, its number `on_primary_fill`, the label `on_surface`. It takes a press, which goes back to it; while the page's work runs, done steps take none.

**Motion:** moving on or back, each bead and label fades to its new look over `short` on `standard`; reduce motion changes them at once. The page's own content turns with it (Make it yours slides its step `space.s24`, shorter than a page's 32, its header and band still).

**Keys:** the beads take no keyboard focus. The page's Back and Esc go back a step, its `Filled` button and Enter on, so the keys stay on the band's button and Enter goes through every step.

*Not built yet: neither library has it.*
