// The Tooltip page: `docs/design/components/tooltip.md`. What a control does, on a card over it
// while the pointer is over it or the keyboard is on it. It isn't neon: it has no line, no glow and
// no states of its own, and fades in and out with its control's lit state, never flickering.

import { chip } from "../../lib/chip.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { keyHints } from "../../lib/keycap.ts";
import { row, stack } from "../../lib/layout.ts";
import { type TipOptions, tooltip, withTooltip } from "../../lib/tooltip.ts";
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

const DESCRIBED =
  "A popover `size.menu` wide: `surface_container`, a soft hairline inside its edge (`outline_variant` at `opacity.outline_soft`), the surface radius (`radius.surface`), `space.s12` inside, and the panel's shadow (`shadow.panel`). Its title is `body_strong` in `on_surface`, its body `small` in `on_surface_variant` `space.s4` under it, both breaking between words in the card's width, and what it holds `space.s8` below. It is never lit: no line, no glow.";

const DECIDED = [
  "Decided, its motion: a tooltip isn't a light, so it never flickers, even though it is a surface appearing: it fades in over `short` with its control lighting up and out over `short` as the control goes back to rest, as a `Hint` does, and as today.",
  "Decided, leaving: it fades out over `short`, as today, not over a panel's `medium`: it goes with the pointer, many times a minute.",
  "Decided, over a disabled control: it still shows under the pointer, as a `Hint` does, so a control that can't be used now still says what it does. Today it shows only when the app asks.",
  "Decided, its shadow by day: the card casts the panel's shadow, `shadow.panel` at 0.55, by day as by night, as today, since a tooltip floats over the page as a panel does; the tucked card's shadow is the only one lighter by day.",
  "Decided, a delay: it shows as its control lights, with no wait, as a `Hint` does and as today, since a tooltip is for the few controls whose words can't say enough.",
];

export default page({
  level: "atoms",
  slug: "tooltip",
  title: "Tooltip",
  summary: "What a control does, on a card over it while the pointer is over it or the keyboard is on it: a title, a line or two under it, and the keys it is about.",
  live: () => row("s24", "end", shownOver(), explained()),
  description: [
    "The live sample on the left is shown over its lit chip; point at the chip on the right, or Tab to it, and its card fades in.",
    "A tooltip explains a control where a `Hint` only names it: a title, the body under it, and what it holds below (the keys it is about, as `KeyHints`). It is for a control whose words can't say enough on their own (the keyboard section's Mac accents chip), never for a sentence the page should show. It shows while the pointer is over its control or the keyboard is on it, which is the control's lit state, fading in over `short` with it and out over `short` as the control goes back to rest. A control it explains shows no `Hint`: the card already says its name.",
    DESCRIBED,
    "It floats `space.s8` above its control, starting where the control starts, on whole device pixels: above, since the controls it explains sit in a toolbar at a panel's foot, and from the control's start, since a card centred on a chip at a toolbar's start would hang past the panel. Where there is no room above (a button in a header at the window's top), it opens `space.s8` below instead, still from the control's start. It is held inside the window, or inside a panel in QML (`within`): a control at the window's end has its card's end on the window's.",
    "It floats over everything: no part of the layout, so it moves nothing as it comes and goes. It never takes the pointer. It shows only while its control shows whole: a control a scroll has cut, or taken out of view, takes its card with it.",
    ...DECIDED,
  ],
  variants: [
    { name: "Above", for: "Every tooltip with room above its control: the card `space.s8` over it, from the control's start.", sample: () => explained({ shown: true }) },
    { name: "Below", for: "A control with no room above it, a header's button at the window's top: the card `space.s8` under it.", sample: () => explained({ side: "below", shown: true }) },
    {
      name: "Held inside the window",
      for: "A control at the window's end, whose card would hang past it from the control's start: the card moves back until its end is the window's, here the sample's end.",
      sample: () => explained({ held: true, shown: true }),
    },
    { name: "Without keys", for: "A card about a control that has no keys of its own: the title and the body.", sample: () => tooltip(shown(TITLE), shown(BODY)) },
    { name: "A title alone", for: "When the title says it all: no body and nothing below.", sample: () => tooltip(shown(TITLE), "") },
  ],
  sizes: doesntApply("a tooltip has one width, `size.menu`, a menu's; it is as tall as what it holds."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "a tooltip takes no pointer and no focus; it shows while its control is lit, and that is its only state.",
      pressed: "it takes no press; pressing its control leaves it as it is.",
      disabled: "over a disabled control it is the same card, shown while the pointer is over the control.",
      selected: "it holds no choice.",
      loading: "its words are known with its control.",
      empty: "it always has a title: a control with nothing to explain has no tooltip.",
      error: "it has no value to be wrong.",
    },
    rows: [
      { name: "With keys", cell: () => card() },
      { name: "Without keys", cell: () => card(false) },
      { name: "A title alone", cell: () => tooltip(shown(TITLE), "") },
    ],
    described: {
      rest: `Shown, while its control is lit. ${DESCRIBED} By day the same roles in daylight colours, with the same shadow.`,
    },
  },
  focus: doesntApply("a tooltip isn't a control and isn't neon: it has no line and no glow, by night or by day. Its control lights as its own page says."),
  content: {
    text: [
      { name: "Its title", words: TITLE, sample: (words: string) => tooltip(words, shown(BODY)) },
      { name: "Its body", words: BODY, sample: (words: string) => tooltip(shown(TITLE), words) },
    ],
    missingPicture: doesntApply("a tooltip has words and keys only."),
    counts: [
      { name: "No keys", for: "The title and the body alone.", sample: () => card(false) },
      { name: "One line of keys", for: "A shortcut the control is about.", sample: () => tooltip(shown(TITLE), shown(BODY), keyHints([["Left Alt+E", "then"], ["E", "types é"]])) },
      { name: "Many lines of keys", for: "Each line a sequence, `space.s8` apart.", sample: () => card() },
    ],
  },
  motion: {
    arrives: [
      {
        name: "Shows",
        what: "Its control lights up, the pointer over it or the keyboard on it: the card fades in over `short` on `standard`, where it will stay. It doesn't move or grow, and it never flickers: it isn't a light.",
        tracks: [{ part: "card", on: "the tooltip", target: ".k-tooltip", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: () => card(),
      },
    ],
    leaves: [
      {
        name: "Hides",
        what: "Its control goes back to rest, or scrolls out of view: the card fades out over `short` on `standard`, where it is.",
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
      { keys: ["Tab", "Shift+Tab"], does: "Focus to its control shows its card; focus moving on hides it, and shows the next control's tooltip or hint." },
      { keys: ["Esc"], does: "Nothing of the tooltip's: it hides only as its control goes back to rest." },
    ],
    order: "No stop of its own: it takes no focus, and is read out as its control's description.",
    lands: "Never: focus lands on its control.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The card", for: "Its width, padding, corners and the gaps inside.", sample: () => card() },
      { name: "Over its control", for: "The gap over its control, from the control's start.", sample: () => explained({ shown: true }) },
    ]),
    measures: [
      { what: "Its width, a menu's.", token: "size-menu" },
      { what: "Its padding.", token: "space-s12" },
      { what: "Its corners.", token: "radius-surface" },
      { what: "The soft hairline inside its edge: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The title's line.", token: "type-body_strong-line" },
      { what: "The body's line.", token: "type-small-line" },
      { what: "The gap between the title and the body.", token: "space-s4" },
      { what: "The gap between its words and its keys, between lines of keys, and between the card and its control.", token: "space-s8" },
      { what: "Its shadow's drop.", token: "shadow-panel-y" },
      { what: "Its shadow's blur.", token: "shadow-panel-blur" },
    ],
    reflow: doesntApply("a tooltip is `size.menu` wide at every window size, its words wrapping inside; at the window's edge it moves back to stay inside."),
  },
});
