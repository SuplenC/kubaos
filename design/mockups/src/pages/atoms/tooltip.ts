// The Tooltip page: `docs/design/components/tooltip.md`. What a control does, on a card over it
// while the pointer is over it or the keyboard is on it. It isn't neon: it has no line, no glow and
// no states of its own, and fades in and out with its control's lit state, never flickering.

import { chip } from "../../lib/chip.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { keyHints } from "../../lib/keycap.ts";
import { row, stack } from "../../lib/layout.ts";
import { keyTooltip, type TipOptions, tooltip, withTooltip } from "../../lib/tooltip.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const TITLE: Words = { short: "Mac accents", long: "Mac accents on Left Alt, for every layout" };
const BODY: Words = {
  short: "Type accents as a Mac does.",
  long: "Type accents as a Mac does: hold Left Alt with the accent's key, then type the letter. Right Alt keeps typing the layout's own third level.",
};

/** The keys the Mac accents card holds: two lines of key hints. */
function accents(): HTMLElement {
  return stack("s8", "start", keyHints([["Left Alt+E", "then"], ["E", "types é"]]), keyHints([["Left Alt+U", "then"], ["O", "types ö"]]));
}

/** The card, its words at the toolbar's length, with or without its keys. */
function card(keys = true, title: string = shown(TITLE), body: string = shown(BODY)): HTMLElement {
  return tooltip(title, body, keys ? accents() : undefined);
}

/** The Mac accents chip in `state`, the control the card explains. */
function control(state: Shown = "rest"): HTMLElement {
  return chip({ label: "Mac accents", kind: "Selected", icon: "keyboard_option_key", press: "press", ...stateOptions(state) });
}

/** The chip with its card, as `options` place it; shown while the chip is lit. */
function explained(options: TipOptions = {}, state: Shown = "rest"): HTMLElement {
  return withTooltip(control(state), card(), options);
}

/** A card shown over its lit chip: what a power change plays on, the card just recolouring. */
function shownOver(): HTMLElement {
  return explained({ shown: true }, "lit");
}

export default page({
  level: "atoms",
  slug: "tooltip",
  title: "Tooltip",
  lede: "What a control does, on a card over it while it is lit: a title, a line or two, its keys.",
  code: { rust: "tooltip::tooltip, tooltip::with_tooltip", qml: "Tooltip" },
  stage: { live: () => row("s48", "end", shownOver(), explained()), hint: "hover the right chip, or Tab to it", daylight: true },
  variants: [
    { name: "Above", for: "with room above: `space.s8` over it, from its start", sample: () => explained({ shown: true }) },
    { name: "Below", for: "no room above: a header's button", sample: () => explained({ side: "below", shown: true }) },
    { name: "Held inside the window", for: "moved back until its end is the window's", sample: () => explained({ held: true, shown: true }) },
    { name: "Without keys", for: "a control with no keys of its own", sample: () => tooltip(shown(TITLE), shown(BODY)) },
    { name: "A title alone", for: "when the title says it all", sample: () => tooltip(shown(TITLE), "") },
    { name: "A name and its key", for: "a button whose key isn't shown: one line", sample: () => keyTooltip("Desktop", "2") },
    { name: "A name and a combination", for: "each key a cap, `space.s4` apart", sample: () => keyTooltip("Full screen", "F") },
  ],
  sizes: doesntApply("one width, `size.menu`; as tall as what it holds."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "it shows while its control is lit",
      pressed: "pressing its control leaves it as it is",
      disabled: "the same card, under the pointer",
      selected: "it holds no choice",
      loading: "its words come with its control",
      empty: "with nothing to explain, there is none",
      error: "it has no value to be wrong",
    },
    rows: [
      { name: "With keys", cell: () => card() },
      { name: "Without keys", cell: () => card(false) },
      { name: "A title alone", cell: () => tooltip(shown(TITLE), "") },
      { name: "A name and its key", cell: () => keyTooltip("Desktop", "2") },
    ],
    looks: {
      rest: "step 4 on `surface_container`, `radius.control`; no line, no glow",
    },
  },
  focus: doesntApply("not a control and not neon: no line and no glow; its control lights."),
  content: {
    text: [
      { name: "Its title", words: TITLE, sample: (words: string) => tooltip(words, shown(BODY)) },
      { name: "Its body", words: BODY, sample: (words: string) => tooltip(shown(TITLE), words) },
      { name: "A name beside its key", words: { short: "Apply", long: "Apply and keep this wallpaper" }, sample: (words: string) => keyTooltip(words, "↵") },
    ],
    missingPicture: doesntApply("words and keys only."),
    counts: [
      { name: "No keys", for: "title and body alone", sample: () => card(false) },
      { name: "One line of keys", for: "a shortcut it is about", sample: () => tooltip(shown(TITLE), shown(BODY), keyHints([["Left Alt+E", "then"], ["E", "types é"]])) },
      { name: "Many lines of keys", for: "a sequence each, `space.s8` apart", sample: () => card() },
    ],
  },
  motion: {
    arrives: [
      {
        name: "Shows",
        what: "It fades in over `short` on `standard` with its control, where it will stay; it never flickers.",
        tracks: [{ part: "card", on: "the tooltip", target: ".k-tooltip", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: () => card(),
      },
    ],
    leaves: [
      {
        name: "Hides",
        what: "It fades out over `short` on `standard`, where it is.",
        tracks: [{ part: "card", on: "the tooltip", target: ".k-tooltip", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: () => card(),
      },
    ],
    between: powerMoves(shownOver),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "focus on its control shows it; moving on hides it" },
      { keys: ["Esc"], does: "nothing of the tooltip's" },
    ],
    order: "No stop of its own: it is read out as its control's description.",
    lands: "Never: focus lands on its control.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The card", for: "width, padding, corners, gaps", sample: () => card() },
      { name: "Over its control", for: "the gap, from its start", sample: () => explained({ shown: true }) },
    ]),
    measures: [
      { part: "Width", token: "size-menu" },
      { part: "Padding", token: "space-s12" },
      { part: "Corner", token: "radius-control" },
      { part: "Rim", token: "stroke-hairline_device_px" },
      { part: "Title line", token: "type-body_strong-line" },
      { part: "Body line", token: "type-small-line" },
      { part: "Title → body", token: "space-s4" },
      { part: "Words → keys, card → control", token: "space-s8" },
      { part: "Name → its key", token: "space-s12" },
      { part: "Between keycaps", token: "space-s4" },
      { part: "Contact blur", token: "elevation-4-contact-blur" },
      { part: "Ambient drop", token: "elevation-4-ambient-y" },
      { part: "Ambient blur", token: "elevation-4-ambient-blur" },
    ],
    reflow: doesntApply("`size.menu` wide at every window size; at the edge it moves back to stay inside."),
  },
});
