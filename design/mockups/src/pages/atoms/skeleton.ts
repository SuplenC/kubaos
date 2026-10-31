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
  name: "What it stands for arrives: it strikes on",
  what: "What the skeleton held the room for arrives: the skeleton goes on that frame, and at night what arrived strikes on in its place through `neon.strike`'s steps over `medium`, as a surface appearing does, since something switching on is a power change. By day it is there at once, as nothing is lit by day; with reduced motion or the flicker off it is there at once too. Nothing moves: it takes exactly the room the skeleton kept.",
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
  what: "Glass of the accent warming up, as a sign's gas does before it lights: every bone's fill rises from `opacity.skeleton` to `skeleton_peak` over `hum` on `standard`, then falls back over `hum`, together, for as long as it shows. At night a faint glow comes in with the peak, `skeleton_glow` fading out over `focus.near`, and goes with it. By day the fill breathes between `skeleton_light` and `skeleton_light_peak`, with no glow.",
  tracks: [
    { part: "glass", on: "every bone", target: ".k-skeleton", property: "--hum", from: 0, to: 1, duration: "hum", curve: "standard" },
    { part: "glass", on: "every bone", target: ".k-skeleton", property: "--hum", from: 1, to: 0, duration: "hum", cue: "hum", curve: "standard" },
  ],
  reduced: [{ part: "glass", on: "every bone", target: ".k-skeleton", property: "--hum", from: 0, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => stack("s24", "start", scanning(true), block(true)),
};

const DESCRIBED =
  "Glass of the accent, `primary` at `opacity.skeleton` (`skeleton_light` by day), with no edge: subtle, so it stands back from the text round it. A `line` is as wide as its room and its style's line tall, at the small radius; a `circle` is an icon size across; a `block` is as wide as its room and a size tall, at a radius role.";

export default page<"loading">({
  level: "atoms",
  slug: "skeleton",
  title: "Skeleton",
  summary: "The shape of what is loading, held in its place in faint glass of the accent that breathes like gas warming up, so what arrives moves nothing.",
  live: () => stack("s24", "start", scanning(), block()),
  description: [
    "Rust `skeleton::skeleton(look, bone)`, QML `Skeleton` (`bone`, `style`, `size`, `radius`). It keeps the room of what is on its way: a Wi-Fi or Bluetooth scan with nothing found yet shows skeleton rows, the system monitor shows skeleton lines for the readings a detailed sample brings, and a `Media` box's placeholder is a block skeleton of its own.",
    DESCRIBED,
    "When what it stands for arrives, the skeleton goes and what arrived takes its room: at night it strikes on through `neon.strike` over `medium`, a power change, as a surface appearing does; by day, with reduced motion and with the flicker off it is there at once. Nothing round it moves.",
    "It breathes while it shows, the glass warming up as a sign's gas does before it lights: every bone's fill rises to `opacity.skeleton_peak` over `hum` and falls back over `hum`, on `standard`, together; at night a faint glow (`skeleton_glow`, out over `focus.near`) comes and goes with the peak. With reduce motion it holds still at rest, and a picture that failed or has no source keeps its placeholder still.",
  ],
  variants: [
    { name: "Line", for: "A line of text on its way, in the style it will be: as wide as its room, its line tall.", sample: () => within("var(--size-menu)", skeleton(LINE)) },
    { name: "Circle", for: "A symbol or a tile on its way: an icon size across.", sample: () => skeleton(CIRCLE) },
    { name: "Block", for: "A box on its way (a thumbnail, a card): as wide as its room, a size tall, at a radius role.", sample: () => block() },
    { name: "A row being scanned", for: "A circle and two lines in a row's places: the shape of the row that will arrive.", sample: () => scanning() },
  ],
  sizes: [
    { name: "A line, by style", for: "As tall as the style's line: `body_strong` 20, `small` 16.", sample: () => within("var(--size-menu)", stack("s8", "start", skeleton(LINE), skeleton({ bone: "line", style: "small" }))) },
    { name: "A circle, by icon size", for: "24 beside a row's title, 48 for a page's one picture.", sample: () => row("s16", "centre", skeleton(CIRCLE), skeleton({ bone: "circle", size: "s48" })) },
    { name: "A block, by size and radius", for: "A size tall at a radius role: here `size.control_lg` at the control radius.", sample: () => block() },
  ],
  states: {
    shown: ["loading"],
    absent: {
      rest: "a skeleton shows only while what it stands for is on its way; when it arrives, it strikes on in the skeleton's place (Motion).",
      lit: "it takes no pointer and no focus, and isn't neon.",
      pressed: "it takes no press: a loading row can't be picked.",
      disabled: "it has nothing to disable.",
      selected: "it holds no choice.",
      empty: "nothing found once the wait is over is an `EmptyState`, not a skeleton.",
      error: "a wait that failed shows its error in place; a picture that failed keeps its placeholder, still.",
    },
    rows: [
      { name: "Line", cell: () => within("var(--size-menu)", skeleton(LINE)) },
      { name: "Circle", cell: () => skeleton(CIRCLE) },
      { name: "Block", cell: () => block() },
      { name: "A row being scanned", cell: () => scanning() },
    ],
    described: {
      loading: `${DESCRIBED} It breathes while it shows. By day the same glass, a touch fainter, with no glow.`,
    },
  },
  focus: doesntApply("a skeleton isn't a control and isn't neon: it has no line and no glow, by night or by day."),
  content: {
    text: doesntApply("a skeleton holds no words: a line skeleton stands for words still on their way."),
    missingPicture: [{ name: "A picture still on its way", for: "A `Media` box's placeholder is a block skeleton in its box, at the box's aspect ratio; a picture that failed keeps it, still.", sample: () => within("var(--size-thumbnail_width)", skeleton({ bone: "block", size: "thumbnail_height", radius: "control" })) }],
    counts: [
      { name: "One", for: "One row on its way.", sample: () => scanning() },
      { name: "Many", for: "A scan with nothing found yet: rows of skeletons one under another.", sample: () => stack("s12", "start", scanning(), scanning(), scanning()) },
    ],
  },
  motion: {
    arrives: [HUM],
    leaves: [STRIKES_ON],
    between: powerMoves(() => stack("s24", "start", scanning(true), block(true))),
  },
  keyboard: doesntApply("a skeleton takes no focus and no key."),
  measures: {
    samples: nonEmpty([
      { name: "A row being scanned", for: "The circle, the two lines and the gaps between them.", sample: () => scanning(true) },
      { name: "Block", for: "Its height and corners.", sample: () => block(true) },
    ]),
    measures: [
      { what: "A row title's line, which is its line skeleton's height.", token: "type-body_strong-line" },
      { what: "A subtitle's line.", token: "type-small-line" },
      { what: "The circle beside two lines.", token: "icon-s24" },
      { what: "A line's corners.", token: "radius-small" },
      { what: "The block's height in the sample.", token: "size-control_lg" },
      { what: "The block's corners in the sample.", token: "radius-control" },
      { what: "The gap between the two lines.", token: "space-s4" },
      { what: "The gap between the circle and the lines.", token: "space-s12" },
    ],
    reflow: {
      described: "A line and a block are as wide as their room at every window size; a circle keeps its size.",
      sample: () => within("100%", stack("s12", "start", row("s12", "centre", skeleton(CIRCLE), filling(stack("s4", "start", skeleton(LINE), skeleton({ bone: "line", style: "small" })))), skeleton(BLOCK))),
    },
  },
});
