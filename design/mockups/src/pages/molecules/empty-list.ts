// The EmptyList page: `docs/design/components/list.md`. A list's well with nothing in it, one centred
// line saying so; and while its rows are still being found, loading rows in their place.

import { shown, type Words } from "../../lib/content.ts";
import { filling, flow, within } from "../../lib/layout.ts";
import { emptyList, type ListRowSpec, list } from "../../lib/list.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { WELL_ROOM } from "./list.ts";

/** The stage's pane, less the room a lit control's glow throws past its edge, which it would otherwise scale down for. */
const ACROSS = "calc(100% - var(--space-s24))";

const LINE: Words = { short: "No devices found", long: "No Bluetooth devices nearby. Put yours in pairing mode to see it here." };
const NETWORKS: Words = { short: "No networks found", long: "No Wi-Fi networks in range. Move closer to one, or plug in a cable." };

/** A scan's rows still being found, each a different share of its words' room. */
const SEARCHING: readonly ListRowSpec[] = [
  { title: "", loading: true, share: 0.6 },
  { title: "", loading: true, share: 0.45 },
  { title: "", loading: true, share: 0.75 },
];

function empty(words: string): HTMLElement {
  return within(WELL_ROOM, emptyList(words));
}

function loading(count = 3): HTMLElement {
  return within(WELL_ROOM, list(SEARCHING.slice(0, count)));
}

const MATRIX = [
  { name: "Bluetooth", words: LINE },
  { name: "Wi-Fi", words: NETWORKS },
] as const;

type MatrixState = "empty" | "loading";

export default page<MatrixState>({
  level: "molecules",
  slug: "empty-list",
  title: "EmptyList",
  lede: "A list's well with nothing in it: one centred line saying so, or rows still being found in their place.",
  code: { rust: "list::empty_list", qml: "EmptyList" },
  stage: {
    live: () => within(ACROSS, filling(flow("s32", empty(shown(LINE)), loading()))),
    hint: "nothing to press: Tab passes it",
    daylight: true,
  },
  variants: [
    { name: "Nothing found", for: "the line, saying what to do where it can", sample: () => empty(shown(LINE)) },
    { name: "Still being found", for: "loading rows in the rows' place", sample: () => loading() },
  ],
  sizes: doesntApply("as tall as its line or loading rows; its width is its place's."),
  states: {
    shown: ["empty", "loading"],
    absent: {
      rest: "with rows it is a `List`",
      lit: "takes no pointer or keyboard",
      pressed: "nothing in it takes a press",
      disabled: "never disabled as a whole",
      selected: "nothing to pick",
      error: "a list that failed is an `ErrorBlock`",
    },
    rows: nonEmpty(MATRIX.map(({ name, words }) => ({ name, cell: (state: MatrixState) => (state === "empty" ? empty(shown(words)) : loading()) }))),
    looks: {
      empty: "the well and one centred `body` line, `on_surface_variant`",
      loading: "skeletons in a tiled two-line row's place, no press",
    },
  },
  focus: doesntApply("it takes no keyboard: Tab passes it."),
  content: {
    text: [
      { name: "Bluetooth", words: LINE, sample: (words: string) => empty(words) },
      { name: "Wi-Fi", words: NETWORKS, sample: (words: string) => empty(words) },
    ],
    missingPicture: doesntApply("it draws no symbol: its line says it all."),
    counts: [
      { name: "None", for: "the line", sample: () => empty(shown(NETWORKS)) },
      { name: "One loading row", for: "a scan expecting little", sample: () => loading(1) },
      { name: "Several loading rows", for: "each a different share of the room", sample: () => loading() },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface; a last row leaving shows the line at once."),
    leaves: doesntApply("found rows take the loading rows' places at once, nothing moving."),
    between: doesntApply("no states to move between; the loading rows breathe as a `Skeleton`."),
  },
  keyboard: doesntApply("it takes no keyboard: Tab passes it."),
  measures: {
    samples: nonEmpty([
      { name: "Nothing found", for: "the well's padding, the line and its room", sample: () => empty(shown(LINE)) },
      { name: "Still being found", for: "a loading row's tile and lines", sample: () => loading(1) },
    ]),
    measures: [
      { part: "Well padding", token: "space-s8" },
      { part: "Line's room above and below", token: "space-s16" },
      { part: "Line", token: "type-body-line" },
      { part: "Loading row", token: "size-row_two_line" },
      { part: "Loading row's tile", token: "size-control_md" },
      { part: "Well corner", token: "radius-content" },
    ],
    reflow: {
      note: "It fills its place, the line centred, ending in an ellipsis where the well is narrower.",
      sample: () => within("100%", emptyList(shown(LINE))),
    },
  },
});
