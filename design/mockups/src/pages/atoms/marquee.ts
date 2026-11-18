// The Marquee page: new, in neither library yet. Words in a room a size token wide that scroll when
// they run past it, so nothing beside them moves when they change. Now playing's song is its first use.

import { iconButton } from "../../lib/icon-button.ts";
import { row, stack } from "../../lib/layout.ts";
import { marquee, marqueeChanging, type MarqueePart } from "../../lib/marquee.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** A run of words in `body`: the first `on_surface`, the rest `on_surface_variant` after a middle dot. */
function words(first: string, second: string): MarqueePart[] {
  return [
    { words: first, style: "body", color: "on_surface" },
    { words: " · ", style: "body", color: "on_surface_variant" },
    { words: second, style: "body", color: "on_surface_variant" },
  ];
}

const SHORT = words("Into the Light", "BerryDeep");
const LONG = words("Everything You Wanted, Nothing You Needed (Live at the Roundhouse)", "The Lanterns");
const OTHER = words("Paper Moons", "Halcyon Drive");

/** Its example room: now playing's. */
const ROOM = "now_playing";

function still(parts: readonly MarqueePart[], at = 0, held = false): HTMLElement {
  return marquee({ parts, room: ROOM, pinned: at, held }).element;
}

/** A marquee that plays, with ⏭ for new words and ⏯ to hold it, live. */
function playing(): HTMLElement {
  const sets = [SHORT, LONG, OTHER];
  let at = 1;
  let held = false;
  const made = marquee({ parts: sets[at]!, room: ROOM });
  const next = iconButton({ icon: "skip_next", label: "New words", kind: "Tonal", size: "Sm" });
  const hold = iconButton({ icon: "pause", label: "Hold", kind: "Tonal", size: "Sm" });
  next.addEventListener("click", () => {
    at = (at + 1) % sets.length;
    made.change(sets[at]!);
  });
  hold.addEventListener("click", () => {
    held = !held;
    made.hold(held);
  });
  return row("s12", "centre", next, hold, made.element);
}

const MOVES: readonly Move[] = [
  {
    name: "Scrolling",
    what: "Words past the room hold `marquee_hold`, then travel `space.s32` per `marquee_pace`, on `linear`.",
    tracks: [{ part: "the words", on: "words past the room", target: ".k-marquee", property: "--marquee-x", from: 0, to: 1, duration: "marquee_pace", cue: "marquee_hold", curve: "linear" }],
    reduced: [{ part: "the words", on: "words past the room", target: ".k-marquee", property: "--marquee-x", from: 0, to: 0, duration: "instant", curve: "linear" }],
    back: "cuts",
    sample: () => still(LONG),
  },
  {
    name: "New words",
    what: "The old fade over `micro`; the new fade in over `short` and rise `space.s8` over `medium`.",
    tracks: [
      { part: "the old words", on: "any words", target: ".k-marquee-track[data-out]", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard_accel" },
      { part: "the new words' fade", on: "any words", target: ".k-marquee-track[data-in]", property: "--marquee-fade", from: 0, to: 1, duration: "short", cue: "micro", curve: "standard" },
      { part: "the new words' rise", on: "any words", target: ".k-marquee-track[data-in]", property: "--marquee-rise", from: 0, to: 1, duration: "medium", cue: "micro", curve: "emphasized_decel" },
    ],
    reduced: [
      { part: "the old words", on: "any words", target: ".k-marquee-track[data-out]", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
      { part: "the new words' rise", on: "any words", target: ".k-marquee-track[data-in]", property: "--marquee-rise", from: 0, to: 1, duration: "instant", curve: "linear" },
    ],
    back: "cuts",
    sample: () => marqueeChanging(SHORT, OTHER, ROOM),
  },
  {
    name: "Held mid-pass",
    what: "The words glide to the nearer start over `medium` on `standard`, and stay.",
    tracks: [{ part: "the words", on: "words past the room", target: ".k-marquee", property: "--marquee-x", from: 3, to: 0, duration: "medium", curve: "standard" }],
    reduced: "same",
    back: "cuts",
    sample: () => still(LONG, 3, true),
  },
];

export default page<"rest">({
  level: "atoms",
  slug: "marquee",
  title: "Marquee",
  lede: "Words in a fixed room that scroll when they run past it, so nothing beside them moves.",
  code: { unbuilt: "New: in neither library yet" },
  stage: {
    live: () => stack("s16", "start", playing(), still(SHORT)),
    hint: "⏭ new words, ⏸ holds them",
    daylight: true,
  },
  variants: [
    { name: "Fits", for: "still at its start, nothing faded", sample: () => still(SHORT) },
    { name: "Runs past", for: "held at its start, the end faded", sample: () => still(LONG) },
    { name: "Mid-pass", for: "cut at both edges, both faded", sample: () => still(LONG, 3) },
  ],
  sizes: doesntApply("its room is a `size` token its owner names: `now_playing`."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "not a control: its owner takes the pointer",
      pressed: "it takes no press",
      disabled: "held: it stops at its start",
      selected: "it holds no choice",
      loading: "its owner shows once its words are known",
      empty: "no words: an empty room",
      error: "words can't be wrong",
    },
    rows: [
      { name: "Fits", cell: () => still(SHORT) },
      { name: "Runs past", cell: () => still(LONG) },
      { name: "Mid-pass", cell: () => still(LONG, 3) },
    ],
    looks: { rest: "its words' own styles and colours, in its room" },
  },
  focus: doesntApply("not a control: no line and no glow."),
  content: {
    text: [{ name: "The words", words: { short: "Halo", long: "Everything You Wanted, Nothing You Needed (Live at the Roundhouse)" }, sample: (first: string) => still(words(first, "BerryDeep")) }],
    missingPicture: doesntApply("it draws no picture."),
    counts: doesntApply("one run of words, in parts of their own styles."),
  },
  motion: {
    arrives: doesntApply("it comes with its owner, its words still at their start."),
    leaves: doesntApply("it goes with its owner."),
    between: [...MOVES, ...powerMoves(() => still(LONG, 3))],
  },
  keyboard: doesntApply("no focus and no key; its words are read out whole."),
  measures: {
    samples: nonEmpty([
      { name: "Runs past", for: "the room, the end's fade", sample: () => still(LONG) },
      { name: "Mid-pass", for: "both fades, the copy behind", sample: () => still(LONG, 3) },
    ]),
    measures: [
      { part: "Room", token: "size-now_playing" },
      { part: "Line", token: "type-body-line" },
      { part: "Copy behind the words", token: "space-s32" },
      { part: "Edge fades", token: "space-s16" },
    ],
    reflow: doesntApply("its room is fixed at every window size."),
  },
});
