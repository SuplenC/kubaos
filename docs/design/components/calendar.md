# Calendar

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Calendar`

Rust `calendar::calendar`; QML `Calendar`.

A month as six weeks, each led by its ISO week number (`caption`, a small control wide), under the weekdays' names (`label`, weekends in `tertiary`). A day is a regular control (40), its disc a small one (32), a pill: today filled with `primary`, the picked day ringed with it in the thin stroke, a holiday dotted in `tertiary`, the days of the months around dimmed to the disabled content's 0.38. Between the arrows (`Text` sm icon buttons) the title, a `subheading` pill, opens the year (`year`): the twelve months three to a row, the one shown filled. The app works the month out (`kubaos-clock` for the shell); the shell's slides a new month in from the side it lies on over `short`, and the wheel moves a month. The clock popup's.

**States:** a day resting, hovered (a holiday's name in a hint) and pressed; today filled, the picked day ringed, a holiday dotted, the days of the months around dimmed; the month and the year views. It has no disabled state.
