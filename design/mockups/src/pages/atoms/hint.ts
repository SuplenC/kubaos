// The Hint page: `docs/design/components/icon-button.md`, "Hint". A control's name in a bubble
// under it while the pointer is over it or the keyboard is on it. It isn't neon: it has no line,
// no glow and no states of its own, and fades in and out with its control's lit state.

import { fixed, shown, type Words } from "../../lib/content.ts";
import { hint, hintRoom, withHint } from "../../lib/hint.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { row } from "../../lib/layout.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const NAME: Words = { short: "Close", long: "Close the panel and go back to the desktop" };

/** A Text icon button at md, lit, its hint under it held as `held` says. */
function lit(words: string, held: "centred" | "end" = "centred"): HTMLElement {
  const control = iconButton({ icon: "close", label: words, kind: "Text", size: "Md", pinned: "lit", hint: false });
  withHint(control, words, { held });
  return hintRoom(control, words, "var(--size-control_md)", held);
}

/** The bubble alone, shown. */
function bubble(words: string): HTMLElement {
  return hint(words, { shown: true });
}

/** Two bubbles side by side: what a power change plays on, the bubble just recolouring. */
function bubbles(): HTMLElement {
  return row("s16", "start", lit(shown(fixed("Settings"))), lit(shown(fixed("Close"))));
}

const DESCRIBED =
  "The inverse colours: `on_surface` under `surface` letters, the `label` style, `space.s8` at its sides and `space.s4` above and below, at the small radius (`radius.small`). By night that is a pale bubble with dark letters, by day a dark one with light letters; it reads on every surface a control sits on, in both modes. It is never lit: no line, no glow.";

export default page({
  level: "atoms",
  slug: "hint",
  title: "Hint",
  summary: "A control's name in a small bubble under it, while the pointer is over it or the keyboard is on it: the label an icon button says out loud, shown.",
  live: () => lit(shown(NAME)),
  description: [
    "A hint names a control that shows no words: an icon button's label, or a pressable chip's `tooltip`. It shows while the pointer is over the control or the keyboard is on it, which is the control's lit state, so the bubble comes and goes as the control lights up and goes back to rest, fading over `short`. Over a disabled control it still shows while the pointer is there, so a disabled icon still says what it is.",
    DESCRIBED,
    "It sits `space.s8` under its control, centred on it, on whole device pixels: half the difference in width, rounded to the nearest device pixel. It is as wide as its words, rounded up to a device pixel, and its padding, and never wraps. A control near the window's edge keeps its bubble whole: the bubble moves aside only as far as it must to stay inside the window.",
    "It floats over everything: no part of the layout, so it moves nothing as it comes and goes, and a surface leaves it room below its controls. It never takes the pointer. It shows only while its control shows whole: a control a scroll has cut, or taken out of view, takes its bubble with it. A control a `Tooltip` explains shows that card instead (`explained`), which already says its name.",
  ],
  variants: [
    { name: "Centred", for: "Under its control, centred on it: every hint whose bubble fits inside the window there.", sample: () => lit(shown(NAME)) },
    {
      name: "Held inside the window",
      for: "A control at the window's end, whose centred bubble would hang past it: the bubble moves aside until its end is the window's, here the end of the sample, and no further.",
      sample: () => lit(shown(NAME), "end"),
    },
  ],
  sizes: doesntApply("a hint has one size: the `label` style and its padding, as wide as its words."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "a hint takes no pointer and no focus; it shows while its control is lit, and that is its only state.",
      pressed: "it takes no press; pressing its control leaves it as it is.",
      disabled: "under a disabled control it is the same bubble, shown while the pointer is over the control.",
      selected: "it holds no choice.",
      loading: "its words are its control's name, there when the control is.",
      empty: "it always has words: a control with no name to show has no hint.",
      error: "it has no value to be wrong.",
    },
    rows: [
      { name: "A short name", cell: () => bubble(shown(fixed("Close"))) },
      { name: "A long name", cell: () => bubble(shown(NAME)) },
    ],
    described: {
      rest: `Shown, while its control is lit. ${DESCRIBED}`,
    },
  },
  focus: doesntApply("a hint isn't a control and isn't neon: it has no line and no glow, by night or by day."),
  content: {
    text: [{ name: "Its control's name", words: NAME, sample: (words: string) => bubble(words) }],
    missingPicture: doesntApply("a hint has words only."),
    counts: doesntApply("one bubble per control, while that control is lit."),
  },
  motion: {
    arrives: [
      {
        name: "Shows",
        what: "Its control lights up, the pointer over it or the keyboard on it: the bubble fades in over `short` on `standard`, with the control, where it will stay. It doesn't move or grow, and it never flickers: it isn't a light.",
        tracks: [{ part: "bubble", on: "the hint", target: ".k-hint", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: () => bubble(shown(NAME)),
      },
    ],
    leaves: [
      {
        name: "Hides",
        what: "Its control goes back to rest, or scrolls out of view: the bubble fades out over `short` on `standard`, where it is.",
        tracks: [{ part: "bubble", on: "the hint", target: ".k-hint", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: () => bubble(shown(NAME)),
      },
    ],
    between: powerMoves(bubbles),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Focus to its control shows its hint; focus moving on hides it, and shows the next control's." },
      { keys: ["Esc"], does: "Nothing of the hint's: it hides only as its control goes back to rest." },
    ],
    order: "No stop of its own: it takes no focus and is read out as its control's name.",
    lands: "Never: focus lands on its control.",
  },
  measures: {
    samples: nonEmpty([{ name: "The bubble", for: "Its words, padding and corners.", sample: () => bubble(shown(fixed("Close"))) }, { name: "Under its control", for: "The gap under its control.", sample: () => lit(shown(fixed("Close"))) }]),
    measures: [
      { what: "Its words' line.", token: "type-label-line" },
      { what: "Its padding at the sides, and its gap under the control.", token: "space-s8" },
      { what: "Its padding above and below.", token: "space-s4" },
      { what: "Its corners.", token: "radius-small" },
    ],
    reflow: doesntApply("a hint is as wide as its words at every window size and never wraps; at the window's edge it moves aside to stay inside."),
  },
});
