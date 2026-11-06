// The Skeleton page: `docs/design/components/skeleton.md`. The shape of what is loading, held in
// its place, so what arrives moves nothing: faint glass of the accent that breathes while it shows,
// warming up as a sign's gas does. It isn't a control.

import { icon } from "../../lib/icon.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { type Bone, skeleton } from "../../lib/skeleton.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const LINE: Bone = { bone: "line", style: "body_strong" };
const CIRCLE: Bone = { bone: "circle", size: "s24" };
const BLOCK: Bone = { bone: "block", size: "control_lg", radius: "control" };

/** A row being scanned, as the specimens draw it: a 24 circle, then two lines `space.s4` apart. */
function scanning(still = false): HTMLElement {
  const options = { still };
  return within(
    "var(--size-column)",
    row("s12", "centre", skeleton(CIRCLE, options), filling(stack("s4", "start", skeleton(LINE, options), skeleton({ bone: "line", style: "small" }, options)))),
  );
}

/** A block in a room `size.menu` wide. */
function block(still = false): HTMLElement {
  return within("var(--size-menu)", skeleton(BLOCK, { still }));
}

/** The row the scan found, in the skeleton's places: a 24 symbol, its title and its subtitle. */
function found(): HTMLElement {
  return within(
    "var(--size-column)",
    row("s12", "centre", icon("wifi", "var(--icon-s24)", "var(--color-on_surface_variant)"), filling(stack("s4", "start", text("Home network", "body_strong"), text("Secured · strong signal", "small", "on_surface_variant")))),
  );
}

/** The skeleton row and the row that takes its place, one over the other in the same room. */
function arriving(): HTMLElement {
  const before = scanning(true);
  before.classList.add("k-arrive-skeleton");
  const after = found();
  after.classList.add("k-arrive-content");
  const room = document.createElement("span");
  room.className = "k-overlap";
  room.append(before, after);
  return room;
}

const ARRIVAL = "what arrived, in the skeleton's place";

/**
 * What the skeleton stands for arriving: a power change, so at night it strikes on in the
 * skeleton's place; by day, with reduced motion and with the flicker off, it is there at once.
 */
const STRIKES_ON: Move = {
  name: "Arrival strikes on",
  what: "The skeleton goes; at night what arrived strikes on over `medium`, by day at once, in exactly its room.",
  tracks: [
    { part: "skeleton", on: "the row being scanned", target: ".k-arrive-skeleton", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "what arrived", on: ARRIVAL, target: ".k-arrive-content", property: "opacity", from: 0, to: 1, duration: "medium", curve: "neon.strike" },
  ],
  daylight: [
    { part: "skeleton", on: "the row being scanned", target: ".k-arrive-skeleton", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "what arrived", on: ARRIVAL, target: ".k-arrive-content", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  reduced: [
    { part: "skeleton", on: "the row being scanned", target: ".k-arrive-skeleton", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "what arrived", on: ARRIVAL, target: ".k-arrive-content", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  unflickered: [
    { part: "skeleton", on: "the row being scanned", target: ".k-arrive-skeleton", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "what arrived", on: ARRIVAL, target: ".k-arrive-content", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "cuts",
  sample: arriving,
};

/** One breath: the glass warming from rest to its peak, then cooling back. */
const HUM: Move = {
  name: "Breathes while it shows",
  what: "The fill rises to `opacity.skeleton_peak` over `hum` and falls back together; at night a faint glow comes with the peak.",
  tracks: [
    { part: "glass", on: "every bone", target: ".k-skeleton", property: "--hum", from: 0, to: 1, duration: "hum", curve: "standard" },
    { part: "glass", on: "every bone", target: ".k-skeleton", property: "--hum", from: 1, to: 0, duration: "hum", cue: "hum", curve: "standard" },
  ],
  reduced: [{ part: "glass", on: "every bone", target: ".k-skeleton", property: "--hum", from: 0, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => stack("s24", "start", scanning(true), block(true)),
};

export default page<"loading">({
  level: "atoms",
  slug: "skeleton",
  title: "Skeleton",
  lede: "The shape of what is loading, held in its place, so what arrives moves nothing.",
  code: { rust: "skeleton::skeleton", qml: "Skeleton" },
  stage: {
    live: () => stack("s24", "start", scanning(), within("var(--size-column)", skeleton(BLOCK))),
    hint: "watch it breathe; reduce motion holds it still",
    daylight: true,
  },
  variants: [
    { name: "Line", for: "text on its way: as wide as its room", sample: () => within("var(--size-menu)", skeleton(LINE)) },
    { name: "Circle", for: "a symbol or a tile: an icon size across", sample: () => skeleton(CIRCLE) },
    { name: "Block", for: "a thumbnail, a card: a size tall", sample: () => block() },
    { name: "A row being scanned", for: "the shape of the row that will arrive", sample: () => scanning() },
  ],
  sizes: [
    { name: "A line, by style", for: "as tall as the style's line", sample: () => within("var(--size-menu)", stack("s8", "start", skeleton(LINE), skeleton({ bone: "line", style: "small" }))) },
    { name: "A circle, by icon size", for: "24 beside a title, 48 for a page's one", sample: () => row("s16", "centre", skeleton(CIRCLE), skeleton({ bone: "circle", size: "s48" })) },
    { name: "A block, by size and radius", for: "here `size.control_lg` at `radius.control`", sample: () => block() },
  ],
  states: {
    shown: ["loading"],
    absent: {
      rest: "it shows only while something is on its way",
      lit: "no pointer and no focus, and not neon",
      pressed: "a loading row can't be picked",
      disabled: "it has nothing to disable",
      selected: "it holds no choice",
      empty: "nothing found is an `EmptyState`",
      error: "a failed wait shows its error in place",
    },
    rows: [
      { name: "Line", cell: () => within("var(--size-menu)", skeleton(LINE)) },
      { name: "Circle", cell: () => skeleton(CIRCLE) },
      { name: "Block", cell: () => block() },
      { name: "A row being scanned", cell: () => scanning() },
    ],
    looks: {
      loading: "`primary` at `opacity.skeleton`, no edge, breathing",
    },
  },
  focus: doesntApply("not a control and not neon: no line and no glow."),
  content: {
    text: doesntApply("no words: a line stands for words on their way."),
    missingPicture: [
      {
        name: "A picture still on its way",
        for: "`Media`'s placeholder; a failed picture keeps it, still",
        sample: () => within("var(--size-thumbnail_width)", skeleton({ bone: "block", size: "thumbnail_height", radius: "control" })),
      },
    ],
    counts: [
      { name: "One", for: "one row on its way", sample: () => scanning() },
      { name: "Many", for: "a scan with nothing found yet", sample: () => stack("s12", "start", scanning(), scanning(), scanning()) },
    ],
  },
  motion: {
    arrives: [HUM],
    leaves: [STRIKES_ON],
    between: powerMoves(() => stack("s24", "start", scanning(true), block(true))),
  },
  keyboard: doesntApply("no focus and no key."),
  measures: {
    samples: nonEmpty([
      { name: "A row being scanned", for: "circle, two lines, gaps", sample: () => scanning(true) },
      { name: "Block", for: "height and corners", sample: () => block(true) },
    ]),
    measures: [
      { part: "Title line", token: "type-body_strong-line" },
      { part: "Subtitle line", token: "type-small-line" },
      { part: "Circle", token: "icon-s24" },
      { part: "Line corner", token: "radius-small" },
      { part: "Block height", token: "size-control_lg" },
      { part: "Block corner", token: "radius-control" },
      { part: "Between the lines", token: "space-s4" },
      { part: "Circle → lines", token: "space-s12" },
    ],
    reflow: {
      note: "A line and a block fill their room at every window size; a circle keeps its size.",
      sample: () => within("100%", stack("s12", "start", row("s12", "centre", skeleton(CIRCLE), filling(stack("s4", "start", skeleton(LINE), skeleton({ bone: "line", style: "small" })))), skeleton(BLOCK))),
    },
  },
});
