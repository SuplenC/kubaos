# Text

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Text`

Rust `text::text(content, style, color)` and `text::line`; QML `Text` (`text`, `style`, `color`).

Words in a type style ([DESIGN.md, "Type"](../../DESIGN.md#type)), picked by what the text is, never by how big it should be, in a colour role. Its box is as wide as the words' advances, rounded up to a device pixel, and as tall as its lines on the style's line height, so a text box is always on the grid. Given less room, it breaks between words, at the same places in both toolkits: `KubaOS.Components` measures words as iced lays them out (`Advances`), so a box is as wide in QML as in iced and parity holds them to a hundredth of a pixel.

- **One line in a box of fixed height** (a row's title, a card's name) ends in an ellipsis where it doesn't fit: `text::line` in Rust, `maximumLines: 1` in QML. In the shell `maximumLines` keeps any number of lines (a window's title, a notification's body), the last one ending in an ellipsis at its room.
- **Coloured runs** (the characters a Command menu query matched): QML's `markup`, the same words as Qt's `StyledText`, drawn over the measured text while it keeps to one line.
- **Tabular figures** come with the style (`label_number`), as "Type" says; a number that ticks in place is a [`Figure`](figure.md).

It has no states of its own: a control's text takes the control's colours, dimmed with it when it is disabled. The geometry check reads every text as a `text` node with its words and its natural size.
