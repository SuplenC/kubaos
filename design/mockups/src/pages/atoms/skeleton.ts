// The Skeleton page: `docs/design/components/skeleton.md`. The shape of what is loading, held in
// its place in the placeholder colour, so what arrives moves nothing. In QML a sheen sweeps across
// it three times as it shows, then it holds still; it isn't a control and isn't neon.

import { filling, row, stack, within } from "../../lib/layout.ts";
import { type Bone, skeleton } from "../../lib/skeleton.ts";
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

/** The sweep: the sheen's middle from a band clear of the left edge to a band clear of the right. */
const SHEEN: Move = {
  name: "Sweeps as it shows",
  what: "As it shows, a sheen of `surface_container_highest` sweeps across it from left to right over `slow` on `standard`, then it rests for `slow`; three times, then it holds still in the plain placeholder colour, so a long wait draws no frames. The sheen's middle runs from 0.4 of the width before the left edge to 0.4 past the right, fading to the placeholder colour 0.4 of the width either side of it.",
  tracks: [{ part: "sheen", on: "every bone", target: ".k-skeleton", property: "--sweep", from: -0.4, to: 1.4, duration: "slow", curve: "standard" }],
  tail: "slow",
  reduced: [{ part: "sheen", on: "every bone", target: ".k-skeleton", property: "--sweep", from: -0.4, to: -0.4, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => stack("s24", "start", scanning(true), block(true)),
};

const DESCRIBED =
  "In the placeholder colour a `Media` box holds, `surface_container_high`. A `line` is as wide as its room and its style's line tall, at the small radius; a `circle` is an icon size across; a `block` is as wide as its room and a size tall, at a radius role.";

export default page<"loading">({
  level: "atoms",
  slug: "skeleton",
  title: "Skeleton",
  summary: "The shape of what is loading, held in its place in the placeholder colour, so what arrives moves nothing.",
  live: () => stack("s24", "start", scanning(), block()),
  description: [
    "Rust `skeleton::skeleton(look, bone)`, QML `Skeleton` (`bone`, `style`, `size`, `radius`). It keeps the room of what is on its way: a Wi-Fi or Bluetooth scan with nothing found yet shows skeleton rows, the system monitor shows skeleton lines for the readings a detailed sample brings, and a `Media` box's placeholder is a block skeleton of its own.",
    DESCRIBED,
    "In QML a sheen of `surface_container_highest` sweeps across it, left to right, over `slow` on `standard`, then rests for `slow`, three times as it shows, then it holds still. The sheen is the shape's own fill, so its corners cut it. With reduce motion it holds still from the start, and a picture that failed or has no source keeps its placeholder still. iced's skeleton holds still.",
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
      rest: "a skeleton shows only while what it stands for is on its way; when it arrives, it takes the skeleton's place.",
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
      loading: `${DESCRIBED} The sheen sweeps three times as it shows, then it holds still. By day the same roles in daylight colours: a pale grey shape, its sheen a step paler.`,
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
    arrives: [SHEEN],
    leaves: doesntApply("what it stands for takes its place on the frame it arrives, in the room the skeleton kept; a live list's new row fades in over `slow`, as `LiveList` says."),
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
