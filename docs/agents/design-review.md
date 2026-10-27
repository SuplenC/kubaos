# Design review

What Kuba checks in every UI change. A design brief includes this list, and a PR is reviewed against it on its rendered screenshots, not only its code, before Kuba sees it. `docs/DESIGN.md` is the spec; this page holds the review points that `just design-check` doesn't measure.

## The checklist

- **No wasted space.** Bands, footers and toolbars are as tall as their content plus the container padding. No empty gutters, no oversized sliders, and list rows sit at their text's natural height. A view that needs scrolling scrolls; it has no "more below" strip.
- **Nothing shifts.** Images and thumbnails hold their final box at a fixed aspect ratio before they load. Numbers that tick keep their width. Content that arrives doesn't push other content around.
- **Key hints are keycaps,** one cap per key, never text on text.
- **Icons are sized to the text beside them** and stay crisp at every scale. Where rows carry icons of different shapes, each icon sits on a tinted tile, so every row reads the same size.
- **Readable in both modes.** Logos, the lock screen and the tray read in light and in dark.
- **Danger looks dangerous.** A destructive step is a properly red card. Its action sits inside that card, held to confirm, so it can't be pressed by accident.
- **Information has its own slot.** For example, a notification shows its app name, date and time in their own places, never folded into the body text.
- **States are told apart.** Done, To do and an action such as Reboot each look different, and the action carries the accent.
- **Small controls stay quiet.** Secondary controls, like a tile's detail arrow, are transparent with a hover colour, not filled.
- **Controls respond.** A slider's fill brightens while it's pressed or dragged.
- **First and last impressions are amazing, not just great.** Kōbō's welcome page and its "That's it" page get a big hero and real animation.

## How to apply it

- **Before building a large visible change** (a new screen, a card's layout, an animation), show Kuba a rough HTML mockup and let him pick, then build it.
- **Before the PR is ready,** a reviewer goes through the changed screenshots against this list and `docs/DESIGN.md`, and the PR lists each visible change as a decision for Kuba, with its before/after shots.
- **A point that becomes measurable** is a `just design-check` rule instead of an entry here.
