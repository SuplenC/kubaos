# StepRailPage

A template: [DESIGN.md, "Levels and where they live"](../../DESIGN.md#levels-and-where-they-live) says what a template is. Its mockup page is the source of truth: Templates, StepRailPage, in `design/mockups/`.

## `StepRailPage`

A step rail beside a page with a footer band, and how it moves between its pages. One template with two hosts: in an app's window (the updater) the rail and the page fill the window, on `surface`; in Kōbō they are its glass panel over the wallpaper and the scrim (`surface` at `opacity.glass` over the blurred wallpaper, the soft hairline, `radius.surface`, `shadow.panel`), inset 32 from a screen two columns wide (16 from a smaller one), centred, and no wider than the rail beside two columns with 40 either side.

**The rail** is `size.rail` (236) wide, 16 in above and below and 12 at its sides, on the well's fill (`surface_container_low` at `opacity.well`): the brand (Kōbō's lockup over its version line) or the window's title, 20, the steps back to back (`Step` in its rail box, a mark at its end), then at its foot the keys the page takes (a keycap column beside what each does) and, in Kōbō, the power button and the battery. A done step shows its check and takes a press to go back, as far as the first step after one that can't be undone, and none while a step runs; the current step is the selected row; a step that failed shows `error`.

**The page** is a clear `Page`: its body takes the height left, then 16 and the footer band, with **no hairline**: Back (`Text`, `arrow_back`) on the left, the one `Filled` button on the right (`arrow_forward` after its label), 16 apart, and a `small` note just before it; while something blocks Continue, its reason after the `error` symbol, Continue disabled, the reason flashing in `error` as Enter is refused and settling over `slow`. A page whose action is in its body, or that runs, has no band.

**Between pages** the new page slides 32 in from the direction of travel and fades in over `long` on `emphasized_decel`, on whole device pixels; the page left goes at once. Reduce motion makes it a fade.

**Keys:** Enter is the page's `Filled` action; Esc goes back a page; ↑ ↓ (k j) move between the rail's pressable steps; Tab goes rail, page, Back, Continue. A page arriving puts the keyboard on its first thing to do.

*Not built yet: neither library has the template; Kōbō draws its rail, panel and footer by hand (`crates/kubaos-kobo/src/view.rs`), and the updater doesn't exist.*
