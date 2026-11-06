// The Hint page: `docs/design/components/icon-button.md`, "Hint". A control's name in a bubble
// under it while the pointer is over it or the keyboard is on it. It isn't neon: it has no line,
// no glow and no states of its own, and fades in and out with its control's lit state.

import { fixed, shown, type Words } from "../../lib/content.ts";
import { type Held, hint, hintRoom, withHint } from "../../lib/hint.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { row } from "../../lib/layout.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const NAME: Words = { short: "Close", long: "Close the panel and go back to the desktop" };

/** A Text icon button at md, lit, its hint under it held as `held` says. */
function lit(words: string, held: Held = "centred"): HTMLElement {
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

/** Icon buttons whose hints show under the pointer, each with its bubble's room. */
function pointAt(name: string, words: string): HTMLElement {
  return hintRoom(iconButton({ icon: name, label: words, kind: "Text", size: "Md" }), words, "var(--size-control_md)");
}

export default page({
  level: "atoms",
  slug: "hint",
  title: "Hint",
  lede: "A control's name in a small bubble under it, while the control is lit.",
  code: { rust: "hint::hint", qml: "Hint" },
  stage: {
    live: () => row("s48", "start", lit(shown(NAME)), pointAt("settings", "Settings"), pointAt("volume_up", "Volume"), pointAt("wifi", "Wi-Fi")),
    hint: "hover a button, or Tab to it",
    daylight: true,
  },
  variants: [
    { name: "Centred", for: "every hint that fits under its control", sample: () => lit(shown(NAME)) },
    { name: "Held to its start", for: "a slider's icon: never past the slider's edge", sample: () => lit(shown(NAME), "start") },
    { name: "Held inside the window", for: "at the window's end, moved aside only as it must", sample: () => lit(shown(NAME), "end") },
  ],
  sizes: doesntApply("one size: `label` and its padding, as wide as its words."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "it shows while its control is lit",
      pressed: "pressing its control leaves it as it is",
      disabled: "the same bubble, under the pointer",
      selected: "it holds no choice",
      loading: "its words come with its control",
      empty: "a control with no name to show has none",
      error: "it has no value to be wrong",
    },
    rows: [
      { name: "A short name", cell: () => bubble(shown(fixed("Close"))) },
      { name: "A long name", cell: () => bubble(shown(NAME)) },
    ],
    looks: {
      rest: "`on_surface` under `surface` letters; no line, no glow",
    },
  },
  focus: doesntApply("not a control and not neon: no line and no glow."),
  content: {
    text: [{ name: "Its control's name", words: NAME, sample: (words: string) => bubble(words) }],
    missingPicture: doesntApply("words only."),
    counts: doesntApply("one bubble per control, while it is lit."),
  },
  motion: {
    arrives: [
      {
        name: "Shows",
        what: "It fades in over `short` on `standard` with its control, where it will stay; it never flickers.",
        tracks: [{ part: "bubble", on: "the hint", target: ".k-hint", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: () => bubble(shown(NAME)),
      },
    ],
    leaves: [
      {
        name: "Hides",
        what: "It fades out over `short` on `standard`, where it is.",
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
      { keys: ["Tab", "Shift+Tab"], does: "focus on its control shows it; moving on hides it" },
      { keys: ["Esc"], does: "nothing of the hint's" },
    ],
    order: "No stop of its own: it is read out as its control's name.",
    lands: "Never: focus lands on its control.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The bubble", for: "words, padding, corners", sample: () => bubble(shown(fixed("Close"))) },
      { name: "Under its control", for: "the gap under it", sample: () => lit(shown(fixed("Close"))) },
    ]),
    measures: [
      { part: "Line", token: "type-label-line" },
      { part: "Sides, gap under the control", token: "space-s8" },
      { part: "Above and below", token: "space-s4" },
      { part: "Corner", token: "radius-row" },
    ],
    reflow: doesntApply("as wide as its words, never wrapping; at the window's edge it moves aside."),
  },
});
